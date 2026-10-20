//! The array-domain emit backend, against the live runtime ABI.
//!
//! `ArrayBackend` implements `spec_core::EmitBackend`. A kind is one row of
//! `KINDS` (its category, picker root, the call its adapter makes, its
//! `CorrectnessSuite`), and `entry_body` is the one template that
//! reads the row and writes the `AlgorithmEntry` registration block spliced
//! into the compiler's `mod __entry_N`. `AbiPaths` names the crates the emitted
//! code targets.
//!
//! What the template gets right:
//!  - Inherent dispatch. A sort is invoked as `<Ty>::sort(arr, logger)`, not
//!    through the legacy `SortAlgo` trait, whose `U` is `Sized` and can't take a
//!    `dyn SortLogger`.
//!  - One adapter shape. `run_dyn(arr, index, logger)` is what every kind
//!    passes to `run_with_input`; a sort ignores the index, a rotation takes its
//!    split, a merge its midpoint, a quick select its target.
//!  - Inherited complexity. `<Ty as HasTimeBounds>::WORST` and its siblings are
//!    read off the type, the way the retired `sort_family!` macro did.
//!
//! LIMITATION: a sort must expose an inherent `sort<T, U: ?Sized +
//! SortLogger<T>>`. A legacy type that implements only the `SortAlgo` trait has
//! no such method, so the emitted `<Ty>::sort` would fail to compile (fail loud,
//! not silent).

use spec_core::{EmitBackend, EntryCtx};

/// The crate paths the emitted code targets. The live ABI is split across three
/// crates (the prototype collapsed them into one `abi` string):
///  - `core`   — `AlgorithmEntry`/`Category`/`RunConfig`/`ALGORITHMS`/
///    `run_with_input`/`correctness::*` (`array_vis_bench_core::bench_registry`)
///  - `traits` — `composable::{HasTimeBounds, HasSpace, HasStability}` and
///    `Complexity` as the entry field type (`array_vis_bench_traits`)
///  - `logger` — `SortLogger` / `NoOpLogger` (`sort_logger`)
///  - `suites` — `CorrectnessSuite` / `SelectSuite` (`array_vis_bench_core::suites`),
///    the per-kind correctness-battery seam the non-sort kinds dispatch through
#[derive(Debug, Clone)]
pub struct AbiPaths {
    pub core: String,
    pub traits: String,
    pub logger: String,
    pub suites: String,
}

impl Default for AbiPaths {
    fn default() -> Self {
        AbiPaths {
            core: "array_vis_bench_core::bench_registry".to_string(),
            traits: "array_vis_bench_traits".to_string(),
            logger: "sort_logger".to_string(),
            suites: "array_vis_bench_core::suites".to_string(),
        }
    }
}

/// One kind of algorithm: every fact about its emitted entry that another kind
/// does not share. Adding a kind is a row here, a `CorrectnessSuite` impl and a
/// `Category` variant in `array_vis_bench_core`.
#[derive(Debug, Clone, Copy)]
struct Kind {
    /// The `Category` variant. The query wrapper hands it to the compiler, which
    /// hands it back verbatim as `EntryCtx::category`.
    category: &'static str,
    /// The picker root the kind's entries nest under.
    root: &'static str,
    /// The body of the `run_dyn(arr, index, logger)` adapter, with `{traits}` and
    /// `{logger}` holes.
    dispatch: &'static str,
    /// The `CorrectnessSuite` impl, a generic over the entry's `Ty`. `None` runs
    /// the sort batteries inline.
    suite: Option<&'static str>,
    /// A `Complexity` constant replacing the inherited worst, best and average.
    time: Option<&'static str>,
    /// A `Complexity` constant replacing the inherited space.
    space: Option<&'static str>,
    /// A literal replacing the inherited stability.
    stable: Option<bool>,
}

const KINDS: &[Kind] = &[
    Kind {
        category: "Sort",
        root: "sorts",
        dispatch: "<Ty>::sort(arr, logger);",
        suite: None,
        time: None,
        space: None,
        stable: None,
    },
    // The target is clamped into the array and an empty array is left alone, the
    // way the standalone quick select registration does.
    Kind {
        category: "QuickSelect",
        root: "quick selects",
        dispatch: "if let Some(last) = arr.len().checked_sub(1) { <Ty as {traits}::QuickSelect>::select(arr, logger, index.min(last)); }",
        suite: Some("SelectSuite"),
        time: None,
        space: None,
        stable: None,
    },
    // `run_rotation` allocates and frees the rotation's scratch buffer.
    Kind {
        category: "Rotation",
        root: "rotations",
        dispatch: "{traits}::role::rotation::run_rotation::<Ty, usize, dyn {logger}::SortLogger<usize>>(arr, index, logger);",
        suite: Some("RotationSuite"),
        time: None,
        space: None,
        stable: None,
    },
    // A small sort only runs on inputs bounded by a compile-time threshold, so its
    // work is constant, O(1), not the leaf's intrinsic in-N scaling (insertion's
    // N^2, say). This is the reasoning a quick sort uses to treat its small sort
    // slot as constant.
    Kind {
        category: "SmallSort",
        root: "small sorts",
        dispatch: "let _ = <Ty as {traits}::SmallSort>::sort(arr, logger);",
        suite: Some("SmallSortSuite"),
        time: Some("CONST"),
        space: None,
        stable: None,
    },
    // A standalone partition is a self pivoting `Pivoted<Scheme, Sel>`; the
    // bounds it returns matter to the battery and not to the visualiser.
    Kind {
        category: "Partition",
        root: "partitions",
        dispatch: "let _ = {traits}::role::partition::run_partition_bounds::<Ty, usize, dyn {logger}::SortLogger<usize>>(arr, logger);",
        suite: Some("PartitionSuite"),
        time: None,
        space: None,
        stable: None,
    },
    // `run_rotation_merge` allocates and frees the merge's scratch buffer.
    Kind {
        category: "Merge",
        root: "merges",
        dispatch: "{traits}::role::merge::run_rotation_merge::<Ty, usize, dyn {logger}::SortLogger<usize>>(arr, index, logger);",
        suite: Some("MergeSuite"),
        time: None,
        space: None,
        stable: None,
    },
    // The in place block roll has uniform O(n) time and O(1) space across
    // backings, which differ only in block width, so the bounds are fixed here
    // and not inherited. Every workflow nets to the identity, so it is stable.
    Kind {
        category: "Scratchspace",
        root: "scratchspaces",
        dispatch: "<Ty as {traits}::role::ScratchspaceRun>::run::<usize, dyn {logger}::SortLogger<usize>>(arr, logger);",
        suite: Some("ScratchspaceSuite"),
        time: Some("N1"),
        space: Some("N1"),
        stable: Some(true),
    },
];

/// The live-ABI emit backend. Carries the [`AbiPaths`] the emitted code targets.
#[derive(Debug, Clone, Default)]
pub struct ArrayBackend {
    pub paths: AbiPaths,
}

impl EmitBackend for ArrayBackend {
    fn entry_body(&self, ctx: &EntryCtx) -> Result<String, String> {
        entry_body(KINDS, &self.paths, ctx)
    }
}

// The registration body for `ctx`, spliced inside the compiler's
// `mod __entry_N { ... pub type Ty = ...; <HERE> }`. It is module interior,
// indented one level (4 spaces) and newline terminated, per the `EmitBackend`
// contract. The compiler treats the category as opaque; `kinds` says which ones
// this backend can emit.
fn entry_body(kinds: &[Kind], paths: &AbiPaths, ctx: &EntryCtx) -> Result<String, String> {
    let kind = kinds.iter().find(|k| k.category == ctx.category).ok_or_else(|| {
        let wired: Vec<String> = kinds.iter().map(|k| format!("`{}`", k.category)).collect();
        format!(
            "spec_emit: no driver for category `{}` yet, wired kinds are {}",
            ctx.category,
            wired.join(", ")
        )
    })?;
    let AbiPaths { core, traits, logger, suites } = paths;
    let category = kind.category;
    let dispatch = kind.dispatch.replace("{traits}", traits).replace("{logger}", logger);
    // `max_input` is read twice from one literal: the entry field bounds the picker
    // and the timing runs, and the battery skips patterns longer than it. A bounded
    // algorithm left uncapped under test would blow up on the battery's oversized
    // inputs.
    let max_input = ctx.max_input.map_or("None".to_string(), |n| format!("Some({n})"));
    // Picker placement: the kind's root, then the catalog `menu` sub-path, and each
    // resolved top level slot as a faceted `(role, value, path)` axis.
    let menu: Vec<String> = [kind.root].into_iter().chain(ctx.menu.iter().map(String::as_str)).map(|seg| format!("{seg:?}")).collect();
    let menu_lit = menu.join(", ");
    let axes_lit: String = ctx
        .axes
        .iter()
        .map(|a| format!("({:?}, {:?}, {:?}), ", a.role, a.value, a.path))
        .collect();
    // The role tagged label template lets the picker render the partial type in the
    // catalog's label syntax.
    let template_lit = format!("{:?}", ctx.label_template);
    let time = |bound: &str| {
        kind.time.map_or_else(
            || format!("<Ty as {traits}::composable::HasTimeBounds>::{bound}"),
            |c| format!("{traits}::Complexity::{c}"),
        )
    };
    let (worst, best, average) = (time("WORST"), time("BEST"), time("AVERAGE"));
    let space = kind.space.map_or_else(
        || format!("<Ty as {traits}::composable::HasSpace>::SPACE"),
        |c| format!("{traits}::Complexity::{c}"),
    );
    let stable = kind.stable.map_or_else(
        || format!("<Ty as {traits}::composable::HasStability>::STABLE"),
        |s| s.to_string(),
    );
    let correct = match kind.suite {
        Some(suite) => {
            format!("<{suites}::{suite}<Ty> as {suites}::CorrectnessSuite>::verify(NAME, {max_input});")
        }
        None => format!(
            "fn sort_noop(arr: &mut [usize], _index: usize, logger: &mut {logger}::NoOpLogger) -> Option<(usize, usize)> {{
            <Ty>::sort(arr, logger);
            None
        }}
        {core}::correctness::sort_battery(sort_noop, NAME, {max_input});
        {core}::correctness::sort_stability_battery(sort_noop, NAME, {stable});"
        ),
    };
    // Flush left template (the leading spaces ARE the generated indentation).
    Ok(format!(
"    const NAME: &str = {label:?};
    #[allow(unused_variables)]
    fn run_dyn(arr: &mut [usize], index: usize, logger: &mut dyn {logger}::SortLogger<usize>) {{
        {dispatch}
    }}
    fn run_default(input_name: &str, config: &{core}::RunConfig, logger: &mut dyn {logger}::SortLogger<usize>) {{
        {core}::run_with_input({core}::Category::{category}, input_name, config, run_dyn, logger);
    }}
    fn run_correct() {{
        {correct}
    }}
    #[linkme::distributed_slice({core}::ALGORITHMS)]
    #[allow(non_upper_case_globals)]
    static ENTRY: {core}::AlgorithmEntry = {core}::AlgorithmEntry {{
        name: NAME,
        category: {core}::Category::{category},
        worst: {worst},
        best: {best},
        average: {average},
        space: {space},
        stable: {stable},
        adaptive: {adaptive},
        nondeterministic: {nondeterministic},
        max_input_size: {max_input},
        menu: &[{menu_lit}],
        axes: &[{axes_lit}],
        label_template: {template_lit},
        run_with_input: run_default,
        run_correctness: run_correct,
    }};
",
        label = ctx.label,
        adaptive = ctx.adaptive,
        nondeterministic = ctx.nondeterministic,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use spec_core::AxisNode;

    const CORE: &str = "array_vis_bench_core::bench_registry";
    const SUITES: &str = "array_vis_bench_core::suites";

    // Each kind with its picker root, the call its adapter makes and its suite.
    const EXPECTED: [(&str, &str, &str, Option<&str>); 7] = [
        ("Sort", "sorts", "<Ty>::sort(arr, logger);", None),
        ("QuickSelect", "quick selects", "<Ty as array_vis_bench_traits::QuickSelect>::select(arr, logger, index.min(last));", Some("SelectSuite")),
        ("Rotation", "rotations", "array_vis_bench_traits::role::rotation::run_rotation::<Ty, usize, dyn sort_logger::SortLogger<usize>>(arr, index, logger);", Some("RotationSuite")),
        ("SmallSort", "small sorts", "let _ = <Ty as array_vis_bench_traits::SmallSort>::sort(arr, logger);", Some("SmallSortSuite")),
        ("Partition", "partitions", "let _ = array_vis_bench_traits::role::partition::run_partition_bounds::<Ty, usize, dyn sort_logger::SortLogger<usize>>(arr, logger);", Some("PartitionSuite")),
        ("Merge", "merges", "array_vis_bench_traits::role::merge::run_rotation_merge::<Ty, usize, dyn sort_logger::SortLogger<usize>>(arr, index, logger);", Some("MergeSuite")),
        ("Scratchspace", "scratchspaces", "<Ty as array_vis_bench_traits::role::ScratchspaceRun>::run::<usize, dyn sort_logger::SortLogger<usize>>(arr, logger);", Some("ScratchspaceSuite")),
    ];

    // A slot-less entry with every facet at its default, so its template equals
    // its label. A test overrides the field it studies.
    fn plain(category: &'static str) -> EntryCtx<'static> {
        EntryCtx { label: "x", category, adaptive: false, nondeterministic: false, max_input: None, axes: &[], menu: &[], label_template: "x" }
    }

    fn body(ctx: EntryCtx) -> String {
        ArrayBackend::default().entry_body(&ctx).unwrap()
    }

    fn sort_body(label: &'static str) -> String {
        body(EntryCtx { label, label_template: label, ..plain("Sort") })
    }

    #[test]
    fn the_table_wires_exactly_the_seven_kinds() {
        let wired: Vec<&str> = KINDS.iter().map(|k| k.category).collect();
        let expected: Vec<&str> = EXPECTED.iter().map(|e| e.0).collect();
        assert_eq!(wired, expected);
    }

    #[test]
    fn every_kind_dispatches_through_its_role_and_verifies_through_its_suite() {
        for (cat, _, call, suite) in EXPECTED {
            let b = body(plain(cat));
            assert!(b.contains(call), "category `{cat}` adapter does not make `{call}`");
            if let Some(suite) = suite {
                let verify = format!("<{SUITES}::{suite}<Ty> as {SUITES}::CorrectnessSuite>::verify(NAME, None);");
                assert!(b.contains(&verify), "category `{cat}` does not verify through `{suite}`");
                assert!(!b.contains("_battery"), "category `{cat}` inlines a battery");
            }
        }
    }

    #[test]
    fn every_kind_runs_its_adapter_under_its_own_category() {
        for (cat, ..) in EXPECTED {
            let b = body(plain(cat));
            assert!(b.contains("fn run_dyn(arr: &mut [usize], index: usize, logger: &mut dyn sort_logger::SortLogger<usize>)"), "category `{cat}` adapter shape");
            assert!(b.contains(&format!("{CORE}::run_with_input({CORE}::Category::{cat}, input_name, config, run_dyn, logger);")), "category `{cat}` driver call");
            assert!(b.contains(&format!("category: {CORE}::Category::{cat},")), "category `{cat}` entry field");
        }
    }

    #[test]
    fn a_sort_dispatches_through_the_inherent_method_not_sortalgo() {
        let b = sort_body("shell sort<sequence: ciura>");
        // the dyn adapter and the battery adapter both call the inherent `<Ty>::sort` ...
        assert_eq!(b.matches("<Ty>::sort(arr, logger);").count(), 2, "expected a dyn + a noop adapter");
        // ... and crucially NOT the legacy SortAlgo trait (Sized U won't take dyn).
        assert!(!b.contains("SortAlgo"), "must not route through the legacy SortAlgo trait");
    }

    #[test]
    fn a_sort_runs_the_real_batteries_with_a_noop_adapter() {
        let b = sort_body("x");
        assert!(b.contains("fn sort_noop(arr: &mut [usize], _index: usize, logger: &mut sort_logger::NoOpLogger) -> Option<(usize, usize)>"));
        assert!(b.contains(&format!("{CORE}::correctness::sort_battery(sort_noop, NAME, None);")));
        assert!(b.contains(&format!("{CORE}::correctness::sort_stability_battery(sort_noop, NAME, <Ty as array_vis_bench_traits::composable::HasStability>::STABLE);")));
    }

    #[test]
    fn a_quick_select_clamps_its_target_and_leaves_an_empty_array() {
        let b = body(plain("QuickSelect"));
        assert!(b.contains("if let Some(last) = arr.len().checked_sub(1) {"));
        assert!(b.contains("index.min(last)"));
    }

    #[test]
    fn registers_into_the_algorithms_slice_with_inherited_complexity() {
        for (cat, ..) in EXPECTED.iter().filter(|e| !["SmallSort", "Scratchspace"].contains(&e.0)) {
            let b = body(plain(cat));
            assert!(b.contains(&format!("#[linkme::distributed_slice({CORE}::ALGORITHMS)]")));
            assert!(b.contains(&format!("static ENTRY: {CORE}::AlgorithmEntry")));
            assert!(b.contains("worst: <Ty as array_vis_bench_traits::composable::HasTimeBounds>::WORST,"), "category `{cat}`");
            assert!(b.contains("space: <Ty as array_vis_bench_traits::composable::HasSpace>::SPACE,"), "category `{cat}`");
            assert!(b.contains("stable: <Ty as array_vis_bench_traits::composable::HasStability>::STABLE,"), "category `{cat}`");
        }
    }

    #[test]
    fn a_small_sort_is_constant_time_and_a_scratchspace_fixes_all_its_bounds() {
        let small = body(plain("SmallSort"));
        assert!(small.contains("worst: array_vis_bench_traits::Complexity::CONST,"));
        assert!(small.contains("average: array_vis_bench_traits::Complexity::CONST,"));
        assert!(small.contains("space: <Ty as array_vis_bench_traits::composable::HasSpace>::SPACE,"));

        let scratch = body(plain("Scratchspace"));
        assert!(scratch.contains("best: array_vis_bench_traits::Complexity::N1,"));
        assert!(scratch.contains("space: array_vis_bench_traits::Complexity::N1,"));
        assert!(scratch.contains("stable: true,"));
    }

    #[test]
    fn a_slotless_entry_sits_at_its_root_with_no_axes() {
        // The flat default: no catalog `menu`, no slots, so the entry carries
        // the "sorts" root and no facets.
        let b = sort_body("shell sort<sequence: knuth>");
        assert!(b.contains(r#"menu: &["sorts"],"#));
        assert!(b.contains("axes: &[],"));
        assert!(b.contains(r#"label_template: "shell sort<sequence: knuth>","#));
        assert!(b.contains(r#"const NAME: &str = "shell sort<sequence: knuth>";"#));
    }

    #[test]
    fn menu_path_and_axes_emit_for_faceted_navigation() {
        // A catalog `menu` sub-path nests the entry under the "sorts" root, and
        // each resolved top-level slot becomes a faceted (role, value) axis.
        let b = body(EntryCtx {
            label: "spec::quick sort<part: left-left, pivot: first>",
            axes: &[
                AxisNode { role: "Partition".into(), value: "left-left".into(), path: "partition".into() },
                AxisNode { role: "Pivot".into(), value: "first".into(), path: "pivot".into() },
            ],
            menu: &["spec".to_string(), "quick sorts".to_string()],
            label_template: "spec::quick sort<part: {Partition}, pivot: {Pivot}>",
            ..plain("Sort")
        });
        assert!(b.contains(r#"menu: &["sorts", "spec", "quick sorts"],"#));
        assert!(b.contains(r#"("Partition", "left-left", "partition")"#));
        assert!(b.contains(r#"("Pivot", "first", "pivot")"#));
        // The role-tagged template is carried verbatim so the picker can
        // render the partial type in the catalog's label syntax.
        assert!(b.contains(r#"label_template: "spec::quick sort<part: {Partition}, pivot: {Pivot}>","#));
    }

    #[test]
    fn every_kind_fills_its_navigation_fields() {
        for (cat, root, ..) in EXPECTED {
            let b = body(EntryCtx {
                axes: &[AxisNode { role: "Role".into(), value: "v".into(), path: "p".into() }],
                menu: &["m".to_string()],
                label_template: "x<{Role}>",
                ..plain(cat)
            });
            assert!(b.contains(&format!(r#"menu: &["{root}", "m"],"#)), "category `{cat}` menu");
            assert!(b.contains(r#"axes: &[("Role", "v", "p"), ],"#), "category `{cat}` axes");
            assert!(b.contains(r#"label_template: "x<{Role}>","#), "category `{cat}` template");
        }
    }

    #[test]
    fn every_kind_threads_the_catalog_facets_to_the_entry_and_the_battery() {
        for (cat, ..) in EXPECTED {
            let b = body(EntryCtx { adaptive: true, nondeterministic: true, max_input: Some(8), ..plain(cat) });
            assert!(b.contains("adaptive: true,"), "category `{cat}` adaptive");
            assert!(b.contains("nondeterministic: true,"), "category `{cat}` nondeterministic");
            assert!(b.contains("max_input_size: Some(8),"), "category `{cat}` cap on the entry");
            assert!(b.contains("NAME, Some(8))"), "category `{cat}` cap on the battery");

            let b = body(plain(cat));
            assert!(b.contains("adaptive: false,") && b.contains("nondeterministic: false,"), "category `{cat}` defaults");
            assert!(b.contains("max_input_size: None,") && b.contains("NAME, None)"), "category `{cat}` uncapped");
        }
    }

    #[test]
    fn every_kind_registers_only_its_entry() {
        // The cap, the flag and the picker data live on the entry, so the only
        // slice an emitted body joins is `ALGORITHMS` and nothing runs before
        // `main`.
        for (cat, ..) in EXPECTED {
            let b = body(EntryCtx { max_input: Some(8), ..plain(cat) });
            assert_eq!(b.matches("distributed_slice").count(), 1, "category `{cat}` joins a second slice");
            assert!(!b.contains("ctor"), "category `{cat}` runs a constructor");
        }
    }

    #[test]
    fn an_unknown_category_names_the_kinds_that_are_wired() {
        let err = ArrayBackend::default().entry_body(&plain("Nonsense")).unwrap_err();
        assert!(err.contains("category `Nonsense`"));
        assert!(err.contains("`Sort`") && err.contains("`Scratchspace`"), "got: {err}");
    }

    #[test]
    fn abi_paths_are_configurable() {
        let backend = ArrayBackend {
            paths: AbiPaths { core: "c".into(), traits: "t".into(), logger: "l".into(), suites: "s".into() },
        };
        let b = backend.entry_body(&plain("Sort")).unwrap();
        assert!(b.contains("&mut l::NoOpLogger"));
        assert!(b.contains("c::run_with_input(c::Category::Sort,"));
        assert!(b.contains("<Ty as t::composable::HasTimeBounds>::WORST"));
        let b = backend.entry_body(&plain("Rotation")).unwrap();
        assert!(b.contains("<s::RotationSuite<Ty> as s::CorrectnessSuite>::verify(NAME, None);"));
    }

    #[test]
    fn a_new_kind_is_a_row_and_a_suite() {
        let toy = Kind {
            category: "Shuffle",
            root: "shuffles",
            dispatch: "<Ty as {traits}::Shuffle>::shuffle(arr, logger, index);",
            suite: Some("ShuffleSuite"),
            time: None,
            space: None,
            stable: None,
        };
        let table = [KINDS, &[toy]].concat();
        let b = entry_body(&table, &AbiPaths::default(), &plain("Shuffle")).unwrap();
        assert!(b.contains("<Ty as array_vis_bench_traits::Shuffle>::shuffle(arr, logger, index);"));
        assert!(b.contains(&format!("<{SUITES}::ShuffleSuite<Ty> as {SUITES}::CorrectnessSuite>::verify(NAME, None);")));
        assert!(b.contains(&format!("category: {CORE}::Category::Shuffle,")));
        assert!(b.contains(r#"menu: &["shuffles"],"#));
        // The production table does not know it.
        assert!(entry_body(KINDS, &AbiPaths::default(), &plain("Shuffle")).is_err());
    }
}
