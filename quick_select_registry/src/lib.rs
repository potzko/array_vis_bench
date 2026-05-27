//! Standalone-quick-select registration: each leaf is a concrete
//! `(QS strategy, PartitionScheme, PivotInput)` combination wrapped as a
//! `Category::QuickSelect` algorithm. The pivot input is baked in; the
//! standalone fn signature is just `(&mut [usize], target, &mut Logger)`.
//!
//! Single-pivot cross-product: 2 (Recursive/Iterative) × 4 partitions
//! (Lomuto/Hoare/ThreeWay/Block) × 5 pivots = 40 leaves.
//! Dual-pivot cross-product: 2 × Yaroslavskiy × 4 dual selectors = 8
//! leaves — registered through the *same* `QuickSelect<P, V>` types as
//! the single-pivot side, with `P = Yaroslavskiy` and `V` a dual-pivot
//! `PivotInput` (`N = 2`). The old `dual_pivot_quick_select_lib` types
//! are gone.
//!
//! This crate has no public API beyond [`LINK_ANCHOR`] — its job is the
//! `#[ctor]` + `#[linkme::distributed_slice]` side-effects that fire
//! when it's linked. Downstream wiring crates reference [`LINK_ANCHOR`]
//! from a `#[used]` static so the linker doesn't drop the object file
//! under `--gc-sections`.

/// Force-link anchor — see module docs.
pub static LINK_ANCHOR: () = ();

use quick_select_lib::{IterativeQuickSelect, RecursiveQuickSelect};
use partition_block::Block;
use partition_hoare::Hoare;
use partition_lomuto::Lomuto;
use partition_three_way::ThreeWay;
use quick_sort_lib::pivot_selectors::{CombinedSelector, NintherDualPivot};
use quick_sort_lib::Yaroslavskiy;
use pivot_first::FirstElement;
use pivot_last::LastElement;
use pivot_median3::MedianOfThree;
use pivot_middle::MiddleElement;
use pivot_ninther::Ninther;

// ── Unified wrapper ──────────────────────────────────────────────────────────
//
// One macro covers both single- and dual-pivot leaves: `$strategy<$part,
// $piv>` instantiates the generic QuickSelect type, `$part_name` /
// `$piv_label` give the menu strings. For single-pivot leaves
// `$piv_label` is the `PivotSelector::NAME`; for dual-pivot it's a
// human-readable combination label ("first / first", "ninther 1/3 +
// 2/3", …). `$part` is `Yaroslavskiy` for the dual-pivot rows.

macro_rules! register_quick_select {
    ($mod:ident, $strategy:ident, $strat_name:expr, $part:ty, $part_name:expr, $piv:ty, $piv_label:expr) => {
        mod $mod {
            use super::*;
            use array_vis_bench_traits::QuickSelect;
            use sort_logger::{NoOpLogger, SortLogger};

            type QS = $strategy<$part, $piv>;

            const NAME: &str = const_format::concatcp!(
                "quick select: ", $strat_name,
                "<", $part_name, ", ", $piv_label, ">",
            );

            fn select_dyn(
                arr: &mut [usize],
                target: usize,
                logger: &mut dyn SortLogger<usize>,
            ) {
                if arr.is_empty() {
                    return;
                }
                let t = target.min(arr.len() - 1);
                <QS as QuickSelect>::select(arr, logger, t)
            }
            fn select_noop(
                arr: &mut [usize],
                target: usize,
                logger: &mut NoOpLogger,
            ) {
                if arr.is_empty() {
                    return;
                }
                let t = target.min(arr.len() - 1);
                <QS as QuickSelect>::select(arr, logger, t)
            }

            fn run_with_input(
                input_name: &str,
                config: &array_vis_bench_core::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                array_vis_bench_core::bench_registry::run_quick_select_with_input(
                    input_name, config, select_dyn, logger,
                );
            }
            fn run_correctness() {
                array_vis_bench_core::bench_registry::correctness::quick_select_battery(
                    select_noop, NAME,
                );
            }

            // Pull worst/best/average/space/stable from the QuickSelect
            // type's compositional impls. Worst flips between O(N) and
            // O(N²) depending on whether the pivot can degenerate.
            #[linkme::distributed_slice(array_vis_bench_core::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: array_vis_bench_core::bench_registry::AlgorithmEntry =
                array_vis_bench_core::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: array_vis_bench_core::bench_registry::Category::QuickSelect,
                    worst: <QS as array_vis_bench_traits::composable::HasTimeBounds>::WORST,
                    best: <QS as array_vis_bench_traits::composable::HasTimeBounds>::BEST,
                    average: <QS as array_vis_bench_traits::composable::HasTimeBounds>::AVERAGE,
                    space: <QS as array_vis_bench_traits::composable::HasSpace>::SPACE,
                    stable: <QS as array_vis_bench_traits::composable::HasStability>::STABLE,
                    adaptive: false,
                    max_input_size: None,
                    run_with_input,
                    run_correctness,
                };

            #[ctor::ctor]
            fn register_path() {
                sort_registry_core::register_sort_path(
                    NAME,
                    <QS as array_vis_bench_traits::composable::HasTimeBounds>::WORST.as_str(),
                    <QS as array_vis_bench_traits::composable::HasStability>::STABLE,
                    &["quick selects", $strat_name, $part_name, $piv_label],
                );
            }
        }
    };
}

// ── Single-pivot cross-product ───────────────────────────────────────────────
//
// {Recursive, Iterative} × {Lomuto, Hoare, ThreeWay, Block} × pivots
register_quick_select!(rec_lomuto_first,    RecursiveQuickSelect, "recursive", Lomuto,   "lomuto",    FirstElement,  "first");
register_quick_select!(rec_lomuto_middle,   RecursiveQuickSelect, "recursive", Lomuto,   "lomuto",    MiddleElement, "middle");
register_quick_select!(rec_lomuto_last,     RecursiveQuickSelect, "recursive", Lomuto,   "lomuto",    LastElement,   "last");
register_quick_select!(rec_lomuto_med3,     RecursiveQuickSelect, "recursive", Lomuto,   "lomuto",    MedianOfThree, "median of 3");
register_quick_select!(rec_lomuto_ninther,  RecursiveQuickSelect, "recursive", Lomuto,   "lomuto",    Ninther,       "ninther");

register_quick_select!(rec_hoare_first,     RecursiveQuickSelect, "recursive", Hoare,    "hoare",     FirstElement,  "first");
register_quick_select!(rec_hoare_middle,    RecursiveQuickSelect, "recursive", Hoare,    "hoare",     MiddleElement, "middle");
register_quick_select!(rec_hoare_last,      RecursiveQuickSelect, "recursive", Hoare,    "hoare",     LastElement,   "last");
register_quick_select!(rec_hoare_med3,      RecursiveQuickSelect, "recursive", Hoare,    "hoare",     MedianOfThree, "median of 3");
register_quick_select!(rec_hoare_ninther,   RecursiveQuickSelect, "recursive", Hoare,    "hoare",     Ninther,       "ninther");

register_quick_select!(rec_3way_first,      RecursiveQuickSelect, "recursive", ThreeWay, "three-way", FirstElement,  "first");
register_quick_select!(rec_3way_middle,     RecursiveQuickSelect, "recursive", ThreeWay, "three-way", MiddleElement, "middle");
register_quick_select!(rec_3way_last,       RecursiveQuickSelect, "recursive", ThreeWay, "three-way", LastElement,   "last");
register_quick_select!(rec_3way_med3,       RecursiveQuickSelect, "recursive", ThreeWay, "three-way", MedianOfThree, "median of 3");
register_quick_select!(rec_3way_ninther,    RecursiveQuickSelect, "recursive", ThreeWay, "three-way", Ninther,       "ninther");

register_quick_select!(rec_block_first,     RecursiveQuickSelect, "recursive", Block,    "block",     FirstElement,  "first");
register_quick_select!(rec_block_middle,    RecursiveQuickSelect, "recursive", Block,    "block",     MiddleElement, "middle");
register_quick_select!(rec_block_last,      RecursiveQuickSelect, "recursive", Block,    "block",     LastElement,   "last");
register_quick_select!(rec_block_med3,      RecursiveQuickSelect, "recursive", Block,    "block",     MedianOfThree, "median of 3");
register_quick_select!(rec_block_ninther,   RecursiveQuickSelect, "recursive", Block,    "block",     Ninther,       "ninther");

register_quick_select!(it_lomuto_first,     IterativeQuickSelect, "iterative", Lomuto,   "lomuto",    FirstElement,  "first");
register_quick_select!(it_lomuto_middle,    IterativeQuickSelect, "iterative", Lomuto,   "lomuto",    MiddleElement, "middle");
register_quick_select!(it_lomuto_last,      IterativeQuickSelect, "iterative", Lomuto,   "lomuto",    LastElement,   "last");
register_quick_select!(it_lomuto_med3,      IterativeQuickSelect, "iterative", Lomuto,   "lomuto",    MedianOfThree, "median of 3");
register_quick_select!(it_lomuto_ninther,   IterativeQuickSelect, "iterative", Lomuto,   "lomuto",    Ninther,       "ninther");

register_quick_select!(it_hoare_first,      IterativeQuickSelect, "iterative", Hoare,    "hoare",     FirstElement,  "first");
register_quick_select!(it_hoare_middle,     IterativeQuickSelect, "iterative", Hoare,    "hoare",     MiddleElement, "middle");
register_quick_select!(it_hoare_last,       IterativeQuickSelect, "iterative", Hoare,    "hoare",     LastElement,   "last");
register_quick_select!(it_hoare_med3,       IterativeQuickSelect, "iterative", Hoare,    "hoare",     MedianOfThree, "median of 3");
register_quick_select!(it_hoare_ninther,    IterativeQuickSelect, "iterative", Hoare,    "hoare",     Ninther,       "ninther");

register_quick_select!(it_3way_first,       IterativeQuickSelect, "iterative", ThreeWay, "three-way", FirstElement,  "first");
register_quick_select!(it_3way_middle,      IterativeQuickSelect, "iterative", ThreeWay, "three-way", MiddleElement, "middle");
register_quick_select!(it_3way_last,        IterativeQuickSelect, "iterative", ThreeWay, "three-way", LastElement,   "last");
register_quick_select!(it_3way_med3,        IterativeQuickSelect, "iterative", ThreeWay, "three-way", MedianOfThree, "median of 3");
register_quick_select!(it_3way_ninther,     IterativeQuickSelect, "iterative", ThreeWay, "three-way", Ninther,       "ninther");

register_quick_select!(it_block_first,      IterativeQuickSelect, "iterative", Block,    "block",     FirstElement,  "first");
register_quick_select!(it_block_middle,     IterativeQuickSelect, "iterative", Block,    "block",     MiddleElement, "middle");
register_quick_select!(it_block_last,       IterativeQuickSelect, "iterative", Block,    "block",     LastElement,   "last");
register_quick_select!(it_block_med3,       IterativeQuickSelect, "iterative", Block,    "block",     MedianOfThree, "median of 3");
register_quick_select!(it_block_ninther,    IterativeQuickSelect, "iterative", Block,    "block",     Ninther,       "ninther");

// ── Dual-pivot cross-product (Yaroslavskiy partition) ────────────────────────
//
// Same `QuickSelect<P, V>` types, with `P = Yaroslavskiy` (N_PIVOTS = 2)
// and a dual-pivot `PivotInput` (N = 2). Dual-pivot is now just the
// "yaroslavskiy" partition row in the menu.
type FirstFirst   = CombinedSelector<FirstElement, FirstElement>;
type MiddleMiddle = CombinedSelector<MiddleElement, MiddleElement>;
type FirstLast    = CombinedSelector<FirstElement, LastElement>;

register_quick_select!(dp_rec_first_first, RecursiveQuickSelect, "recursive", Yaroslavskiy, "yaroslavskiy", FirstFirst,       "first / first");
register_quick_select!(dp_rec_mid_mid,     RecursiveQuickSelect, "recursive", Yaroslavskiy, "yaroslavskiy", MiddleMiddle,     "middle / middle");
register_quick_select!(dp_rec_first_last,  RecursiveQuickSelect, "recursive", Yaroslavskiy, "yaroslavskiy", FirstLast,        "first / last");
register_quick_select!(dp_rec_ninther,     RecursiveQuickSelect, "recursive", Yaroslavskiy, "yaroslavskiy", NintherDualPivot, "ninther 1/3 + 2/3");

register_quick_select!(dp_it_first_first,  IterativeQuickSelect, "iterative", Yaroslavskiy, "yaroslavskiy", FirstFirst,       "first / first");
register_quick_select!(dp_it_mid_mid,      IterativeQuickSelect, "iterative", Yaroslavskiy, "yaroslavskiy", MiddleMiddle,     "middle / middle");
register_quick_select!(dp_it_first_last,   IterativeQuickSelect, "iterative", Yaroslavskiy, "yaroslavskiy", FirstLast,        "first / last");
register_quick_select!(dp_it_ninther,      IterativeQuickSelect, "iterative", Yaroslavskiy, "yaroslavskiy", NintherDualPivot, "ninther 1/3 + 2/3");
