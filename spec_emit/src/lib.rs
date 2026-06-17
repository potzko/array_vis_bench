//! The array-domain emit backend, against the **live** runtime ABI.
//!
//! This is the real counterpart to the prototype's `avb_emit`: it implements
//! [`spec_core::EmitBackend`] and produces, for each solved sort, the exact
//! `AlgorithmEntry` registration block that `sort_registry_macro::sort_family!`
//! emits today — so a `spec_core`-generated leaf is link-time indistinguishable
//! from a hand-registered one.
//!
//! What the audit forced this backend to get right (vs. the prototype's stub):
//!  - **Inherent dispatch.** Sorts are invoked as `<Ty>::sort(arr, logger)`, not
//!    `<Ty as SortAlgo<…, dyn …>>::sort` — the real `SortAlgo` is the legacy
//!    indirect path and its `U` is `Sized` (can't take `dyn SortLogger`).
//!  - **Two adapters.** A `dyn`-logger adapter for the vis path
//!    (`run_sort_with_input`) and a concrete `&mut NoOpLogger` adapter for the
//!    correctness batteries (`sort_battery` takes `fn(&mut [usize], &mut NoOpLogger)`).
//!  - **Real correctness.** `correctness::sort_battery` + `sort_stability_battery`
//!    (the stub's `assert_sorts` does not exist).
//!  - **Split ABI paths.** The live ABI lives across four crates, so the single
//!    `abi` prefix is replaced by [`AbiPaths`].
//!
//! Complexity is still *inherited* from the type (`<Ty as HasTimeBounds>::WORST`,
//! …) — the audit confirmed the live macro and the hand-written shell
//! registration both do exactly this, so that axis ports unchanged.
//!
//! LIMITATION (current scope): this backend supports only sorts exposing an
//! inherent `sort<T, U: ?Sized + SortLogger<T>>` method — the modern
//! `direct_sort = true` shape (shell, quick, merge, heap, …). A legacy type that
//! implements ONLY the `SortAlgo` trait (`direct_sort = false`, e.g.
//! `comb_sort_lib::CombSortRatio`) has no inherent `sort`, so the emitted
//! `<Ty>::sort` would fail to compile (fail-loud, not silent). Supporting those
//! needs a `direct_sort` catalog facet that branches the adapters the way
//! `sort_family!` does (indirect → `SortAlgo<…, NoOpLogger>` for the battery, a
//! no-op for the dyn path) — deferred until such a family is migrated.

use spec_core::{EmitBackend, EntryCtx};

/// The crate paths the emitted code targets. The live ABI is split across four
/// crates (the prototype collapsed them into one `abi` string):
///  - `core`   — `AlgorithmEntry`/`Category`/`RunConfig`/`ALGORITHMS`/
///    `run_sort_with_input`/`correctness::*` (`array_vis_bench_core::bench_registry`)
///  - `traits` — `composable::{HasTimeBounds, HasSpace, HasStability}` and
///    `Complexity` as the entry field type (`array_vis_bench_traits`)
///  - `nav`    — `register_sort_variant` (`sort_registry_core`)
///  - `logger` — `SortLogger` / `NoOpLogger` (`sort_logger`)
///  - `suites` — `CorrectnessSuite` / `SelectSuite` (`array_vis_bench_core::suites`),
///    the per-kind correctness-battery seam the non-sort kinds dispatch through
#[derive(Debug, Clone)]
pub struct AbiPaths {
    pub core: String,
    pub traits: String,
    pub nav: String,
    pub logger: String,
    pub suites: String,
}

impl Default for AbiPaths {
    fn default() -> Self {
        AbiPaths {
            core: "array_vis_bench_core::bench_registry".to_string(),
            traits: "array_vis_bench_traits".to_string(),
            nav: "sort_registry_core".to_string(),
            logger: "sort_logger".to_string(),
            suites: "array_vis_bench_core::suites".to_string(),
        }
    }
}

/// The live-ABI emit backend. Carries the [`AbiPaths`] the emitted code targets.
#[derive(Debug, Clone, Default)]
pub struct ArrayBackend {
    pub paths: AbiPaths,
}

impl EmitBackend for ArrayBackend {
    fn entry_body(&self, ctx: &EntryCtx) -> Result<String, String> {
        // A component with no declared category defaults to Sort (the domain's
        // call — the compiler treats `category` as opaque). Each first-class
        // KIND has its own per-kind body; the correctness battery lives behind
        // `CorrectnessSuite` (one impl per role-trait-backed kind) so adding a
        // kind is a new body + a `CorrectnessSuite` impl, not a battery `match`.
        match ctx.category.unwrap_or("Sort") {
            "Sort" => Ok(self.sort_entry_body(ctx)),
            "QuickSelect" => Ok(self.quick_select_entry_body(ctx)),
            other => Err(format!(
                "spec_emit: no driver for category `{other}` yet — wired kinds are \
                 `Sort` and `QuickSelect`; Partition/Merge/Rotation/SmallSort (whose \
                 live contracts differ) come next, each as a `CorrectnessSuite` impl"
            )),
        }
    }
}

impl ArrayBackend {
    /// The `Category::Sort` registration body, spliced inside the compiler's
    /// `mod __entry_N { … pub type Ty = …; <HERE> }`. Mirrors the direct-sort
    /// arm of `sort_family!` ([sort_family.rs] lines ~759-840): two type-erased
    /// adapters, the vis-path `run_default`, the battery-path `run_correct`, the
    /// `ALGORITHMS` entry with inherited complexity, and the menu `#[ctor]`.
    ///
    /// Returned source is module-interior, indented one level (4 spaces) and
    /// newline-terminated, per the [`EmitBackend`] contract.
    fn sort_entry_body(&self, ctx: &EntryCtx) -> String {
        let AbiPaths { core, traits, nav, logger, suites: _ } = &self.paths;
        let max_input = match ctx.max_input {
            Some(n) => format!("Some({n})"),
            None => "None".to_string(),
        };
        // `max_input` is a HARD contract bound (the type cannot accept inputs
        // larger than `n`), so it must cap BOTH independent knobs:
        //   - `max_input_size` — the interactive picker cap (`read_size_with_cap`
        //     in main). `sort_family!` can't set this (always `None`); only
        //     hand-written registrations (e.g. small-sorts) do.
        //   - `SORT_TEST_CAPS` — the correctness-battery cap, read via
        //     `max_n_for_tests`, NOT from `max_input_size`. This is the field
        //     `sort_family!` drives (from its separate `max_n_for_tests`).
        // Emitting only the picker cap would leave the battery uncapped, and a
        // bounded sort would blow up on the battery's oversized random inputs —
        // the per-entry-cap trap the project guards against. (A "slow but
        // unbounded" sort wanting ONLY the test cap is a separate facet, deferred
        // — no migrated family needs it yet.)
        let cap_static = match ctx.max_input {
            Some(n) => format!(
"    #[linkme::distributed_slice({core}::SORT_TEST_CAPS)]
    #[allow(non_upper_case_globals)]
    static TEST_CAP: (&'static str, usize) = (NAME, {n});
"
            ),
            None => String::new(),
        };
        // A nondeterministic-trace sort (e.g. randomised gaps) registers into
        // `NONDETERMINISTIC_ALGOS` so the determinism property-check skips it —
        // the data-driven equivalent of `register_nondeterministic!`.
        let nondet_static = if ctx.nondeterministic {
            format!(
"    #[linkme::distributed_slice({core}::NONDETERMINISTIC_ALGOS)]
    #[allow(non_upper_case_globals)]
    static NONDET: &'static str = NAME;
"
            )
        } else {
            String::new()
        };
        // Picker placement: nest under the category root (`"sorts"`) followed by
        // the catalog `menu` sub-path, and expose each resolved top-level slot as
        // a faceted `(role, value)` axis — the same shape `sort_family!` emits, so
        // the spec entries navigate faithfully instead of as flat leaves.
        let mut menu_lit = String::from("\"sorts\"");
        for seg in ctx.menu {
            menu_lit.push_str(&format!(", {seg:?}"));
        }
        let axes_lit: String = ctx
            .axes
            .iter()
            .map(|a| format!("({:?}, {:?}, {:?}), ", a.role, a.value, a.path))
            .collect();
        // The role-tagged label template lets the picker render the partial type
        // in the catalog's label syntax. Emitted as a string literal.
        let template_lit = format!("{:?}", ctx.label_template);
        // Flush-left template (the leading spaces ARE the generated indentation).
        format!(
"    const NAME: &str = {label:?};
    fn sort_noop(arr: &mut [usize], logger: &mut {logger}::NoOpLogger) {{
        <Ty>::sort(arr, logger);
    }}
    fn sort_dyn(arr: &mut [usize], logger: &mut dyn {logger}::SortLogger<usize>) {{
        <Ty>::sort(arr, logger);
    }}
    fn run_default(input_name: &str, config: &{core}::RunConfig, logger: &mut dyn {logger}::SortLogger<usize>) {{
        {core}::run_sort_with_input(input_name, config, sort_dyn, logger);
    }}
    fn run_correct() {{
        {core}::correctness::sort_battery(sort_noop, NAME);
        {core}::correctness::sort_stability_battery(sort_noop, NAME, <Ty as {traits}::composable::HasStability>::STABLE);
    }}
{cap_static}{nondet_static}    #[linkme::distributed_slice({core}::ALGORITHMS)]
    #[allow(non_upper_case_globals)]
    static ENTRY: {core}::AlgorithmEntry = {core}::AlgorithmEntry {{
        name: NAME,
        category: {core}::Category::Sort,
        worst: <Ty as {traits}::composable::HasTimeBounds>::WORST,
        best: <Ty as {traits}::composable::HasTimeBounds>::BEST,
        average: <Ty as {traits}::composable::HasTimeBounds>::AVERAGE,
        space: <Ty as {traits}::composable::HasSpace>::SPACE,
        stable: <Ty as {traits}::composable::HasStability>::STABLE,
        adaptive: {adaptive},
        max_input_size: {max_input},
        run_with_input: run_default,
        run_correctness: run_correct,
    }};
    #[ctor::ctor]
    #[allow(non_snake_case)]
    fn register() {{ {nav}::register_sort_variant_structured(NAME, &[{menu_lit}], &[{axes_lit}], {template_lit}); }}
",
            // `core`, `traits`, `nav`, `logger`, `max_input`, `cap_static`,
            // `menu_lit`, `axes_lit`, `template_lit` are captured inline; only
            // these two rename.
            label = ctx.label,
            adaptive = ctx.adaptive,
        )
    }

    /// The `Category::QuickSelect` registration body — the first non-sort
    /// first-class kind. Mirrors the structure of `sort_entry_body`, but the
    /// ABI is the `QuickSelect` *role trait* (not an inherent `sort`), so:
    ///  - the dyn adapter is `(arr, target, logger)` calling
    ///    `<Ty as QuickSelect>::select` (empty-guard + target-clamp, matching
    ///    the standalone `quick_select_registry`);
    ///  - the vis path uses `run_quick_select_with_input`;
    ///  - correctness goes through `CorrectnessSuite::verify` — the per-kind
    ///    battery trait — so no battery call is inlined here.
    /// Entries nest under the `"quick selects"` picker root.
    fn quick_select_entry_body(&self, ctx: &EntryCtx) -> String {
        let AbiPaths { core, traits, nav, logger, suites } = &self.paths;
        let max_input = match ctx.max_input {
            Some(n) => format!("Some({n})"),
            None => "None".to_string(),
        };
        // The quick-select battery uses fixed-size shapes (it doesn't consult
        // `max_n_for_tests`), so a cap here only bounds the interactive picker.
        // Emitted for uniformity; inert unless a leaf declares `max_input`.
        let cap_static = match ctx.max_input {
            Some(n) => format!(
"    #[linkme::distributed_slice({core}::SORT_TEST_CAPS)]
    #[allow(non_upper_case_globals)]
    static TEST_CAP: (&'static str, usize) = (NAME, {n});
"
            ),
            None => String::new(),
        };
        let mut menu_lit = String::from("\"quick selects\"");
        for seg in ctx.menu {
            menu_lit.push_str(&format!(", {seg:?}"));
        }
        let axes_lit: String = ctx
            .axes
            .iter()
            .map(|a| format!("({:?}, {:?}, {:?}), ", a.role, a.value, a.path))
            .collect();
        let template_lit = format!("{:?}", ctx.label_template);
        format!(
"    const NAME: &str = {label:?};
    fn select_dyn(arr: &mut [usize], target: usize, logger: &mut dyn {logger}::SortLogger<usize>) {{
        if arr.is_empty() {{ return; }}
        let t = target.min(arr.len() - 1);
        <Ty as {traits}::QuickSelect>::select(arr, logger, t);
    }}
    fn run_default(input_name: &str, config: &{core}::RunConfig, logger: &mut dyn {logger}::SortLogger<usize>) {{
        {core}::run_quick_select_with_input(input_name, config, select_dyn, logger);
    }}
    fn run_correct() {{
        <{suites}::SelectSuite<Ty> as {suites}::CorrectnessSuite>::verify(NAME);
    }}
{cap_static}    #[linkme::distributed_slice({core}::ALGORITHMS)]
    #[allow(non_upper_case_globals)]
    static ENTRY: {core}::AlgorithmEntry = {core}::AlgorithmEntry {{
        name: NAME,
        category: {core}::Category::QuickSelect,
        worst: <Ty as {traits}::composable::HasTimeBounds>::WORST,
        best: <Ty as {traits}::composable::HasTimeBounds>::BEST,
        average: <Ty as {traits}::composable::HasTimeBounds>::AVERAGE,
        space: <Ty as {traits}::composable::HasSpace>::SPACE,
        stable: <Ty as {traits}::composable::HasStability>::STABLE,
        adaptive: {adaptive},
        max_input_size: {max_input},
        run_with_input: run_default,
        run_correctness: run_correct,
    }};
    #[ctor::ctor]
    #[allow(non_snake_case)]
    fn register() {{ {nav}::register_sort_variant_structured(NAME, &[{menu_lit}], &[{axes_lit}], {template_lit}); }}
",
            label = ctx.label,
            adaptive = ctx.adaptive,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sort_body(label: &str) -> String {
        // A slot-less variant: its template equals its label (no holes).
        ArrayBackend::default()
            .entry_body(&EntryCtx { label, category: Some("Sort"), adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &[], label_template: label })
            .unwrap()
    }

    #[test]
    fn dispatches_through_the_inherent_method_not_sortalgo() {
        let b = sort_body("shell sort<sequence: ciura>");
        // both adapters call the inherent `<Ty>::sort` …
        assert_eq!(b.matches("<Ty>::sort(arr, logger);").count(), 2, "expected a noop + a dyn adapter");
        // … and crucially NOT the legacy SortAlgo trait (Sized U won't take dyn).
        assert!(!b.contains("SortAlgo"), "must not route through the legacy SortAlgo trait");
    }

    #[test]
    fn emits_two_distinct_adapters() {
        let b = sort_body("x");
        assert!(b.contains("fn sort_noop(arr: &mut [usize], logger: &mut sort_logger::NoOpLogger)"));
        assert!(b.contains("fn sort_dyn(arr: &mut [usize], logger: &mut dyn sort_logger::SortLogger<usize>)"));
    }

    #[test]
    fn vis_path_uses_run_sort_with_input_with_the_dyn_adapter() {
        let b = sort_body("x");
        assert!(b.contains("array_vis_bench_core::bench_registry::run_sort_with_input(input_name, config, sort_dyn, logger);"));
    }

    #[test]
    fn correctness_path_uses_the_real_batteries_with_the_noop_adapter() {
        let b = sort_body("x");
        assert!(b.contains("array_vis_bench_core::bench_registry::correctness::sort_battery(sort_noop, NAME);"));
        assert!(b.contains("correctness::sort_stability_battery(sort_noop, NAME, <Ty as array_vis_bench_traits::composable::HasStability>::STABLE)"));
        // the stub's invented assert_sorts must be gone.
        assert!(!b.contains("assert_sorts"));
    }

    #[test]
    fn registers_into_real_algorithms_with_inherited_complexity() {
        let b = sort_body("x");
        assert!(b.contains("#[linkme::distributed_slice(array_vis_bench_core::bench_registry::ALGORITHMS)]"));
        assert!(b.contains("static ENTRY: array_vis_bench_core::bench_registry::AlgorithmEntry"));
        assert!(b.contains("worst: <Ty as array_vis_bench_traits::composable::HasTimeBounds>::WORST,"));
        assert!(b.contains("space: <Ty as array_vis_bench_traits::composable::HasSpace>::SPACE,"));
        assert!(b.contains("category: array_vis_bench_core::bench_registry::Category::Sort,"));
    }

    #[test]
    fn menu_registration_targets_sort_registry_core() {
        // The flat default: no catalog `menu`, no slots → register at the
        // "sorts" root with no facets (same as before menu/axes existed).
        let b = sort_body("shell sort<sequence: knuth>");
        assert!(b.contains(
            r#"sort_registry_core::register_sort_variant_structured(NAME, &["sorts"], &[], "shell sort<sequence: knuth>")"#
        ));
        assert!(b.contains(r#"const NAME: &str = "shell sort<sequence: knuth>";"#));
    }

    #[test]
    fn menu_path_and_axes_emit_for_faceted_navigation() {
        // A catalog `menu` sub-path nests the entry under the "sorts" root, and
        // each resolved top-level slot becomes a faceted (role, value) axis.
        let b = ArrayBackend::default()
            .entry_body(&EntryCtx {
                label: "spec::quick sort<part: left-left, pivot: first>",
                category: Some("Sort"),
                adaptive: false,
                nondeterministic: false,
                max_input: None,
                axes: &[
                    spec_core::AxisNode { role: "Partition".into(), value: "left-left".into(), path: "partition".into() },
                    spec_core::AxisNode { role: "Pivot".into(), value: "first".into(), path: "pivot".into() },
                ],
                menu: &["spec".to_string(), "quick sorts".to_string()],
                label_template: "spec::quick sort<part: {Partition}, pivot: {Pivot}>",
            })
            .unwrap();
        assert!(b.contains(r#"register_sort_variant_structured(NAME, &["sorts", "spec", "quick sorts"], &["#));
        assert!(b.contains(r#"("Partition", "left-left", "partition")"#));
        assert!(b.contains(r#"("Pivot", "first", "pivot")"#));
        // The role-tagged template is registered verbatim so the picker can
        // render the partial type in the catalog's label syntax.
        assert!(b.contains(r#""spec::quick sort<part: {Partition}, pivot: {Pivot}>")"#));
    }

    #[test]
    fn catalog_facets_thread_through() {
        let b = ArrayBackend::default()
            .entry_body(&EntryCtx { label: "x", category: None, adaptive: true, nondeterministic: false, max_input: Some(32), axes: &[], menu: &[], label_template: "x" })
            .unwrap();
        // category=None defaults to Sort in the backend, not the compiler.
        assert!(b.contains("category: array_vis_bench_core::bench_registry::Category::Sort,"));
        assert!(b.contains("adaptive: true,"));
        assert!(b.contains("max_input_size: Some(32),"));
    }

    #[test]
    fn declared_max_input_also_caps_the_correctness_battery() {
        // A ceiling caps both the picker (`max_input_size`) AND the battery
        // (a `SORT_TEST_CAPS` static) — emitting only the former would leave a
        // bounded sort uncapped under test.
        let capped = ArrayBackend::default()
            .entry_body(&EntryCtx { label: "slow", category: Some("Sort"), adaptive: false, nondeterministic: false, max_input: Some(64), axes: &[], menu: &[], label_template: "slow" })
            .unwrap();
        assert!(capped.contains("max_input_size: Some(64),"));
        assert!(capped
            .contains("#[linkme::distributed_slice(array_vis_bench_core::bench_registry::SORT_TEST_CAPS)]"));
        assert!(capped.contains("static TEST_CAP: (&'static str, usize) = (NAME, 64);"));

        // No ceiling → no test cap registered (matches `sort_family!`).
        let uncapped = sort_body("fast");
        assert!(uncapped.contains("max_input_size: None,"));
        assert!(!uncapped.contains("SORT_TEST_CAPS"));
    }

    #[test]
    fn nondeterministic_flag_registers_the_determinism_opt_out() {
        let nd = ArrayBackend::default()
            .entry_body(&EntryCtx { label: "random shell sort<uniform>", category: Some("Sort"), adaptive: false, nondeterministic: true, max_input: Some(1000), axes: &[], menu: &[], label_template: "random shell sort<uniform>" })
            .unwrap();
        assert!(nd.contains(
            "#[linkme::distributed_slice(array_vis_bench_core::bench_registry::NONDETERMINISTIC_ALGOS)]"
        ));
        assert!(nd.contains("static NONDET: &'static str = NAME;"));

        // A deterministic sort emits no opt-out.
        assert!(!sort_body("classic").contains("NONDETERMINISTIC_ALGOS"));
    }

    #[test]
    fn non_sort_categories_are_not_wired_yet() {
        // QuickSelect is now wired (see `quick_select_*` tests); the remaining
        // kinds still error loudly until each gets a body + `CorrectnessSuite`.
        for cat in ["Partition", "Merge", "Rotation", "SmallSort"] {
            let err = ArrayBackend::default()
                .entry_body(&EntryCtx { label: "x", category: Some(cat), adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &[], label_template: "x" })
                .unwrap_err();
            assert!(err.contains(&format!("category `{cat}`")));
        }
    }

    fn quick_select_body(label: &str) -> String {
        ArrayBackend::default()
            .entry_body(&EntryCtx { label, category: Some("QuickSelect"), adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &["recursive".to_string()], label_template: label })
            .unwrap()
    }

    #[test]
    fn quick_select_dispatches_through_the_role_trait_select() {
        let b = quick_select_body("quick select: recursive<left-left pointer, first>");
        // the dyn adapter calls the QuickSelect role trait with the clamped target …
        assert!(b.contains("<Ty as array_vis_bench_traits::QuickSelect>::select(arr, logger, t);"));
        assert!(b.contains("let t = target.min(arr.len() - 1);"));
        // … the vis path uses the quick-select runner …
        assert!(b.contains("array_vis_bench_core::bench_registry::run_quick_select_with_input(input_name, config, select_dyn, logger);"));
        // … and never the sort inherent method or sort battery.
        assert!(!b.contains("<Ty>::sort"));
        assert!(!b.contains("sort_battery"));
    }

    #[test]
    fn quick_select_correctness_goes_through_the_suite_trait() {
        let b = quick_select_body("quick select: iterative<dual pivot, first / first>");
        assert!(b.contains(
            "<array_vis_bench_core::suites::SelectSuite<Ty> as array_vis_bench_core::suites::CorrectnessSuite>::verify(NAME);"
        ));
    }

    #[test]
    fn quick_select_registers_with_the_quick_select_category_under_its_root() {
        let b = quick_select_body("quick select: recursive<block, ninther>");
        assert!(b.contains("category: array_vis_bench_core::bench_registry::Category::QuickSelect,"));
        // nests under the "quick selects" picker root + the driver's menu sub-path.
        assert!(b.contains(r#"register_sort_variant_structured(NAME, &["quick selects", "recursive"], "#));
        // complexity still inherited from the type.
        assert!(b.contains("worst: <Ty as array_vis_bench_traits::composable::HasTimeBounds>::WORST,"));
    }

    #[test]
    fn abi_paths_are_configurable() {
        let backend = ArrayBackend {
            paths: AbiPaths { core: "c".into(), traits: "t".into(), nav: "n".into(), logger: "l".into(), suites: "s".into() },
        };
        let b = backend
            .entry_body(&EntryCtx { label: "x", category: Some("Sort"), adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &[], label_template: "x" })
            .unwrap();
        assert!(b.contains("&mut l::NoOpLogger"));
        assert!(b.contains("c::run_sort_with_input"));
        assert!(b.contains("<Ty as t::composable::HasTimeBounds>::WORST"));
        assert!(b.contains("n::register_sort_variant"));
    }
}
