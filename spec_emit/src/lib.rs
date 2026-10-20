//! The array-domain emit backend, against the **live** runtime ABI.
//!
//! This is the real counterpart to the prototype's `avb_emit`: it implements
//! [`spec_core::EmitBackend`] and produces, for each solved sort, the exact
//! `AlgorithmEntry` registration block the (now-retired) `sort_family!` macro
//! used to emit — so a `spec_core`-generated leaf is link-time indistinguishable
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
        // The compiler treats the category as opaque. Each first-class KIND has
        // its own per-kind body; the correctness battery lives behind
        // `CorrectnessSuite` (one impl per role-trait-backed kind) so adding a
        // kind is a new body + a `CorrectnessSuite` impl, not a battery `match`.
        match ctx.category {
            "Sort" => Ok(self.sort_entry_body(ctx)),
            "QuickSelect" => Ok(self.quick_select_entry_body(ctx)),
            "Rotation" => Ok(self.rotation_entry_body(ctx)),
            "SmallSort" => Ok(self.small_sort_entry_body(ctx)),
            "Partition" => Ok(self.partition_entry_body(ctx)),
            "Merge" => Ok(self.merge_entry_body(ctx)),
            "Scratchspace" => Ok(self.scratchspace_entry_body(ctx)),
            other => Err(format!(
                "spec_emit: no driver for category `{other}` yet — wired kinds are \
                 `Sort`, `QuickSelect`, `Rotation`, `SmallSort`, `Partition`, `Merge`, `Scratchspace`"
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
        // `max_input` is read twice from one literal: the entry field bounds the
        // picker and the timing runs, and `sort_battery` skips patterns longer
        // than it. A bounded sort left uncapped under test would blow up on the
        // battery's oversized random inputs.
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
        {core}::correctness::sort_battery(sort_noop, NAME, {max_input});
        {core}::correctness::sort_stability_battery(sort_noop, NAME, <Ty as {traits}::composable::HasStability>::STABLE);
    }}
    #[linkme::distributed_slice({core}::ALGORITHMS)]
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
        nondeterministic: {nondeterministic},
        max_input_size: {max_input},
        run_with_input: run_default,
        run_correctness: run_correct,
    }};
    #[ctor::ctor]
    #[allow(non_snake_case)]
    fn register() {{ {nav}::register_sort_variant_structured(NAME, &[{menu_lit}], &[{axes_lit}], {template_lit}); }}
",
            // `core`, `traits`, `nav`, `logger`, `max_input`, `menu_lit`,
            // `axes_lit`, `template_lit` are captured inline; only these three
            // rename.
            label = ctx.label,
            adaptive = ctx.adaptive,
            nondeterministic = ctx.nondeterministic,
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
    #[linkme::distributed_slice({core}::ALGORITHMS)]
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
        nondeterministic: false,
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

    /// The `Category::Scratchspace` registration body. The in-place block-roll
    /// has uniform `O(n)` time / `O(1)` space across backings (they differ only in
    /// block width), so complexity is fixed here rather than inherited from a
    /// trait. Correctness goes through `ScratchspaceSuite` (round-trip identity).
    /// Entries nest under the `"scratchspaces"` picker root, faceted by backing.
    fn scratchspace_entry_body(&self, ctx: &EntryCtx) -> String {
        let AbiPaths { core, traits, nav, logger, suites } = &self.paths;
        let mut menu_lit = String::from("\"scratchspaces\"");
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
    fn run_dyn(arr: &mut [usize], logger: &mut dyn {logger}::SortLogger<usize>) {{
        <Ty as {traits}::role::ScratchspaceRun>::run::<usize, dyn {logger}::SortLogger<usize>>(arr, logger);
    }}
    fn run_default(input_name: &str, config: &{core}::RunConfig, logger: &mut dyn {logger}::SortLogger<usize>) {{
        {core}::run_scratchspace_with_input(input_name, config, run_dyn, logger);
    }}
    fn run_correct() {{
        <{suites}::ScratchspaceSuite<Ty> as {suites}::CorrectnessSuite>::verify(NAME);
    }}
    #[linkme::distributed_slice({core}::ALGORITHMS)]
    #[allow(non_upper_case_globals)]
    static ENTRY: {core}::AlgorithmEntry = {core}::AlgorithmEntry {{
        name: NAME,
        category: {core}::Category::Scratchspace,
        worst: {traits}::Complexity::N1,
        best: {traits}::Complexity::N1,
        average: {traits}::Complexity::N1,
        space: {traits}::Complexity::N1,
        stable: true,
        adaptive: {adaptive},
        nondeterministic: false,
        max_input_size: None,
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

    /// The `Category::Merge` registration body. Merges have no role trait in the
    /// traits crate — `RotationMerge` lives in the `merge_sort_lib` family — so,
    /// like sorts, this inlines `merge_battery` directly (no `CorrectnessSuite`,
    /// which would force `array_vis_bench_core` to depend on a family crate). The
    /// shared `run_rotation_merge` one-shot allocates/frees the merge's scratch.
    /// Entries nest under the `"merges"` picker root, faceted by the rotation.
    fn merge_entry_body(&self, ctx: &EntryCtx) -> String {
        let AbiPaths { core, traits, nav, logger, suites: _ } = &self.paths;
        let max_input = match ctx.max_input {
            Some(n) => format!("Some({n})"),
            None => "None".to_string(),
        };
        let mut menu_lit = String::from("\"merges\"");
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
    fn merge_dyn(arr: &mut [usize], mid: usize, logger: &mut dyn {logger}::SortLogger<usize>) {{
        merge_sort_lib::rotation_merge::run_rotation_merge::<Ty, usize, dyn {logger}::SortLogger<usize>>(arr, mid, logger);
    }}
    fn merge_noop(arr: &mut [usize], mid: usize, logger: &mut {logger}::NoOpLogger) {{
        merge_sort_lib::rotation_merge::run_rotation_merge::<Ty, usize, {logger}::NoOpLogger>(arr, mid, logger);
    }}
    fn run_default(input_name: &str, config: &{core}::RunConfig, logger: &mut dyn {logger}::SortLogger<usize>) {{
        {core}::run_merge_with_input(input_name, config, merge_dyn, logger);
    }}
    fn run_correct() {{
        {core}::correctness::merge_battery(merge_noop, NAME);
    }}
    #[linkme::distributed_slice({core}::ALGORITHMS)]
    #[allow(non_upper_case_globals)]
    static ENTRY: {core}::AlgorithmEntry = {core}::AlgorithmEntry {{
        name: NAME,
        category: {core}::Category::Merge,
        worst: <Ty as {traits}::composable::HasTimeBounds>::WORST,
        best: <Ty as {traits}::composable::HasTimeBounds>::BEST,
        average: <Ty as {traits}::composable::HasTimeBounds>::AVERAGE,
        space: <Ty as {traits}::composable::HasSpace>::SPACE,
        stable: <Ty as {traits}::composable::HasStability>::STABLE,
        adaptive: {adaptive},
        nondeterministic: false,
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

    /// The `Category::SmallSort` registration body. ABI is the [`SmallSort`]
    /// role trait: `sort_dyn(arr, logger)` calls `<Ty as SmallSort>::sort`
    /// (the mutation bool is discarded), the vis path uses
    /// `run_small_sort_with_input`, and correctness goes through `SmallSortSuite`
    /// (which feeds the leaf's `THRESHOLD` to the battery). A small sort is only
    /// correct up to its threshold, so `max_input` (set per leaf to the
    /// threshold) caps the interactive picker. Entries nest
    /// under the `"small sorts"` picker root.
    fn small_sort_entry_body(&self, ctx: &EntryCtx) -> String {
        let AbiPaths { core, traits, nav, logger, suites } = &self.paths;
        let max_input = match ctx.max_input {
            Some(n) => format!("Some({n})"),
            None => "None".to_string(),
        };
        let mut menu_lit = String::from("\"small sorts\"");
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
    fn sort_dyn(arr: &mut [usize], logger: &mut dyn {logger}::SortLogger<usize>) {{
        let _ = <Ty as {traits}::SmallSort>::sort(arr, logger);
    }}
    fn run_default(input_name: &str, config: &{core}::RunConfig, logger: &mut dyn {logger}::SortLogger<usize>) {{
        {core}::run_small_sort_with_input(input_name, config, sort_dyn, logger);
    }}
    fn run_correct() {{
        <{suites}::SmallSortSuite<Ty> as {suites}::CorrectnessSuite>::verify(NAME);
    }}
    #[linkme::distributed_slice({core}::ALGORITHMS)]
    #[allow(non_upper_case_globals)]
    static ENTRY: {core}::AlgorithmEntry = {core}::AlgorithmEntry {{
        name: NAME,
        category: {core}::Category::SmallSort,
        // A small sort runs only on inputs bounded by a compile-time THRESHOLD,
        // so its work is constant — O(1), not the leaf's intrinsic in-N scaling
        // (e.g. insertion's N²). This is the same reasoning QuickSort uses to
        // treat its small-sort slot as CONST; the standalone entry matches it.
        worst: {traits}::Complexity::CONST,
        best: {traits}::Complexity::CONST,
        average: {traits}::Complexity::CONST,
        space: <Ty as {traits}::composable::HasSpace>::SPACE,
        stable: <Ty as {traits}::composable::HasStability>::STABLE,
        adaptive: {adaptive},
        nondeterministic: false,
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

    /// The `Category::Partition` registration body. A standalone partition is a
    /// self-pivoting `Pivoted<Scheme, Sel>` (a `PartitionScheme`); one step runs
    /// via the shared `run_partition_bounds` (vis path drops the bounds; the
    /// `PartitionSuite` battery uses them). Entries nest under the `"partitions"`
    /// picker root, faceted by the `scheme`/`pivot` slots.
    fn partition_entry_body(&self, ctx: &EntryCtx) -> String {
        let AbiPaths { core, traits, nav, logger, suites } = &self.paths;
        let max_input = match ctx.max_input {
            Some(n) => format!("Some({n})"),
            None => "None".to_string(),
        };
        let mut menu_lit = String::from("\"partitions\"");
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
    fn partition_dyn(arr: &mut [usize], logger: &mut dyn {logger}::SortLogger<usize>) {{
        let _ = {traits}::role::partition::run_partition_bounds::<Ty, usize, dyn {logger}::SortLogger<usize>>(arr, logger);
    }}
    fn run_default(input_name: &str, config: &{core}::RunConfig, logger: &mut dyn {logger}::SortLogger<usize>) {{
        {core}::run_partition_with_input(input_name, config, partition_dyn, logger);
    }}
    fn run_correct() {{
        <{suites}::PartitionSuite<Ty> as {suites}::CorrectnessSuite>::verify(NAME);
    }}
    #[linkme::distributed_slice({core}::ALGORITHMS)]
    #[allow(non_upper_case_globals)]
    static ENTRY: {core}::AlgorithmEntry = {core}::AlgorithmEntry {{
        name: NAME,
        category: {core}::Category::Partition,
        worst: <Ty as {traits}::composable::HasTimeBounds>::WORST,
        best: <Ty as {traits}::composable::HasTimeBounds>::BEST,
        average: <Ty as {traits}::composable::HasTimeBounds>::AVERAGE,
        space: <Ty as {traits}::composable::HasSpace>::SPACE,
        stable: <Ty as {traits}::composable::HasStability>::STABLE,
        adaptive: {adaptive},
        nondeterministic: false,
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

    /// The `Category::Rotation` registration body. ABI is the [`Rotation`]
    /// role trait: the adapter is `(arr, split, logger)` calling the shared
    /// `run_rotation` one-shot (which allocates/frees the rotation's scratch),
    /// the vis path uses `run_rotation_with_input`, and correctness goes
    /// through `RotationSuite` via [`CorrectnessSuite`]. Entries nest under the
    /// `"rotations"` picker root.
    fn rotation_entry_body(&self, ctx: &EntryCtx) -> String {
        let AbiPaths { core, traits, nav, logger, suites } = &self.paths;
        let max_input = match ctx.max_input {
            Some(n) => format!("Some({n})"),
            None => "None".to_string(),
        };
        let mut menu_lit = String::from("\"rotations\"");
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
    fn rotate_dyn(arr: &mut [usize], split: usize, logger: &mut dyn {logger}::SortLogger<usize>) {{
        {traits}::role::rotation::run_rotation::<Ty, usize, dyn {logger}::SortLogger<usize>>(arr, split, logger);
    }}
    fn run_default(input_name: &str, config: &{core}::RunConfig, logger: &mut dyn {logger}::SortLogger<usize>) {{
        {core}::run_rotation_with_input(input_name, config, rotate_dyn, logger);
    }}
    fn run_correct() {{
        <{suites}::RotationSuite<Ty> as {suites}::CorrectnessSuite>::verify(NAME);
    }}
    #[linkme::distributed_slice({core}::ALGORITHMS)]
    #[allow(non_upper_case_globals)]
    static ENTRY: {core}::AlgorithmEntry = {core}::AlgorithmEntry {{
        name: NAME,
        category: {core}::Category::Rotation,
        worst: <Ty as {traits}::composable::HasTimeBounds>::WORST,
        best: <Ty as {traits}::composable::HasTimeBounds>::BEST,
        average: <Ty as {traits}::composable::HasTimeBounds>::AVERAGE,
        space: <Ty as {traits}::composable::HasSpace>::SPACE,
        stable: <Ty as {traits}::composable::HasStability>::STABLE,
        adaptive: {adaptive},
        nondeterministic: false,
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
            .entry_body(&EntryCtx { label, category: "Sort", adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &[], label_template: label })
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
        assert!(b.contains("array_vis_bench_core::bench_registry::correctness::sort_battery(sort_noop, NAME, None);"));
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
                category: "Sort",
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
            .entry_body(&EntryCtx { label: "x", category: "Sort", adaptive: true, nondeterministic: false, max_input: Some(32), axes: &[], menu: &[], label_template: "x" })
            .unwrap();
        assert!(b.contains("adaptive: true,"));
        assert!(b.contains("max_input_size: Some(32),"));
    }

    #[test]
    fn declared_max_input_is_passed_to_the_correctness_battery() {
        let capped = ArrayBackend::default()
            .entry_body(&EntryCtx { label: "slow", category: "Sort", adaptive: false, nondeterministic: false, max_input: Some(64), axes: &[], menu: &[], label_template: "slow" })
            .unwrap();
        assert!(capped.contains("max_input_size: Some(64),"));
        assert!(capped.contains("correctness::sort_battery(sort_noop, NAME, Some(64));"));

        let uncapped = sort_body("fast");
        assert!(uncapped.contains("max_input_size: None,"));
        assert!(uncapped.contains("correctness::sort_battery(sort_noop, NAME, None);"));
    }

    #[test]
    fn every_kind_registers_only_its_entry() {
        // The cap and the flag live on the entry, so the only slice an emitted
        // body joins is `ALGORITHMS`.
        for cat in ["Sort", "QuickSelect", "Rotation", "SmallSort", "Partition", "Merge", "Scratchspace"] {
            let b = ArrayBackend::default()
                .entry_body(&EntryCtx { label: "x", category: cat, adaptive: false, nondeterministic: false, max_input: Some(8), axes: &[], menu: &[], label_template: "x" })
                .unwrap();
            assert_eq!(b.matches("distributed_slice").count(), 1, "category `{cat}` joins a second slice");
        }
    }

    #[test]
    fn nondeterministic_flag_sets_the_entry_field() {
        let nd = ArrayBackend::default()
            .entry_body(&EntryCtx { label: "random shell sort<uniform>", category: "Sort", adaptive: false, nondeterministic: true, max_input: Some(1000), axes: &[], menu: &[], label_template: "random shell sort<uniform>" })
            .unwrap();
        assert!(nd.contains("nondeterministic: true,"));
        assert_eq!(nd.matches("distributed_slice").count(), 1, "the flag is not a side list");

        // A deterministic sort says so on the entry too.
        assert!(sort_body("classic").contains("nondeterministic: false,"));
    }

    #[test]
    fn every_kind_fills_the_nondeterministic_field() {
        for cat in ["Sort", "QuickSelect", "Rotation", "SmallSort", "Partition", "Merge", "Scratchspace"] {
            let b = ArrayBackend::default()
                .entry_body(&EntryCtx { label: "x", category: cat, adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &[], label_template: "x" })
                .unwrap();
            assert!(b.contains("nondeterministic: false,"), "category `{cat}` leaves the field unset");
        }
    }

    #[test]
    fn all_first_class_kinds_are_wired() {
        // Every kind now has a body; only an UNKNOWN category errors loudly.
        for cat in ["Sort", "QuickSelect", "Rotation", "SmallSort", "Partition", "Merge", "Scratchspace"] {
            assert!(
                ArrayBackend::default()
                    .entry_body(&EntryCtx { label: "x", category: cat, adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &[], label_template: "x" })
                    .is_ok(),
                "category `{cat}` should be wired"
            );
        }
        let err = ArrayBackend::default()
            .entry_body(&EntryCtx { label: "x", category: "Nonsense", adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &[], label_template: "x" })
            .unwrap_err();
        assert!(err.contains("category `Nonsense`"));
    }

    #[test]
    fn rotation_body_uses_the_rotation_abi_and_suite() {
        let b = ArrayBackend::default()
            .entry_body(&EntryCtx { label: "reversal", category: "Rotation", adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &[], label_template: "reversal" })
            .unwrap();
        assert!(b.contains("run_rotation::<Ty"));
        assert!(b.contains("run_rotation_with_input(input_name, config, rotate_dyn, logger);"));
        assert!(b.contains("RotationSuite<Ty>"));
        assert!(b.contains("category: array_vis_bench_core::bench_registry::Category::Rotation,"));
    }

    fn quick_select_body(label: &str) -> String {
        ArrayBackend::default()
            .entry_body(&EntryCtx { label, category: "QuickSelect", adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &["recursive".to_string()], label_template: label })
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
            .entry_body(&EntryCtx { label: "x", category: "Sort", adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &[], label_template: "x" })
            .unwrap();
        assert!(b.contains("&mut l::NoOpLogger"));
        assert!(b.contains("c::run_sort_with_input"));
        assert!(b.contains("<Ty as t::composable::HasTimeBounds>::WORST"));
        assert!(b.contains("n::register_sort_variant"));
    }
}
