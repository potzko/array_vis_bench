//! Standalone-quick-select registration: each leaf is a concrete
//! `(QS strategy, Partition, PivotSelector)` (single-pivot) or
//! `(QS strategy, DualPivotSelector)` (dual-pivot) combination,
//! wrapped as a `Category::QuickSelect` algorithm. The pivot selector
//! is baked in; the standalone fn signature is just
//! `(&mut [usize], target, &mut Logger)`.
//!
//! Single-pivot cross-product: 2 (Recursive/Iterative) × 4 partitions
//! (Lomuto/Hoare/ThreeWay/Block) × 5 pivots = 40 leaves.
//! Dual-pivot cross-product: 2 × 4 selectors = 8 leaves.

use crate::sorts::quick_selects::quick_select::{IterativeQuickSelect, RecursiveQuickSelect};
use crate::sorts::quick_selects::dual_pivot_quick_select::{
    IterativeDualPivotQuickSelect, RecursiveDualPivotQuickSelect,
};
use crate::sorts::quick_sorts::partitions::{Block, Hoare, Lomuto, ThreeWay};
use crate::sorts::quick_sorts::pivot_selectors::{
    CombinedSelector, FirstElement, LastElement, MedianOfThree, MiddleElement, NintherDualPivot,
    Ninther,
};

// ── Single-pivot wrapper ─────────────────────────────────────────────────────

macro_rules! register_quick_select_single {
    ($mod:ident, $strategy:ident, $strat_name:expr, $part:ty, $piv:ty) => {
        mod $mod {
            use super::*;
            use crate::sorts::quick_selects::quick_select::QuickSelect;
            use crate::sorts::quick_sorts::partitions::PartitionScheme;
            use crate::sorts::quick_sorts::pivot_selectors::PivotSelector;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            type QS = $strategy<$part, $piv>;

            const NAME: &str = const_format::concatcp!(
                "quick select: ", $strat_name,
                "<", <$part as PartitionScheme>::NAME,
                ", ", <$piv as PivotSelector>::NAME, ">",
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
                config: &crate::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                crate::bench_registry::run_quick_select_with_input(
                    input_name, config, select_dyn, logger,
                );
            }
            fn run_correctness() {
                crate::bench_registry::correctness::quick_select_battery(
                    select_noop, NAME,
                );
            }

            #[linkme::distributed_slice(crate::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: crate::bench_registry::AlgorithmEntry =
                crate::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: crate::bench_registry::Category::QuickSelect,
                    big_o: "O(N)",
                    stable: false,
                    max_input_size: None,
                    run_with_input,
                    run_correctness,
                };

            #[ctor::ctor]
            fn register_path() {
                sort_registry_core::register_sort_path(
                    NAME,
                    "O(N)",
                    false,
                    &[
                        "quick selects",
                        $strat_name,
                        <$part as PartitionScheme>::NAME,
                        <$piv as PivotSelector>::NAME,
                    ],
                );
            }

            #[cfg(test)]
            mod qs_test {
                #[test]
                fn correctness() {
                    crate::bench_registry::test_helpers::check_sort_subprocess_assert(
                        &super::ENTRY,
                        crate::bench_registry::test_helpers::DEFAULT_TIMEOUT,
                    );
                }
            }
        }
    };
}

// ── Dual-pivot wrapper ───────────────────────────────────────────────────────

macro_rules! register_quick_select_dual {
    ($mod:ident, $strategy:ident, $strat_name:expr, $dps:ty, $dps_name:expr) => {
        mod $mod {
            use super::*;
            use crate::sorts::quick_selects::quick_select::QuickSelect;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            type QS = $strategy<$dps>;

            const NAME: &str = const_format::concatcp!(
                "quick select: ", $strat_name,
                " (dual pivot)<", $dps_name, ">",
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
                config: &crate::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                crate::bench_registry::run_quick_select_with_input(
                    input_name, config, select_dyn, logger,
                );
            }
            fn run_correctness() {
                crate::bench_registry::correctness::quick_select_battery(
                    select_noop, NAME,
                );
            }

            #[linkme::distributed_slice(crate::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: crate::bench_registry::AlgorithmEntry =
                crate::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: crate::bench_registry::Category::QuickSelect,
                    big_o: "O(N)",
                    stable: false,
                    max_input_size: None,
                    run_with_input,
                    run_correctness,
                };

            #[ctor::ctor]
            fn register_path() {
                sort_registry_core::register_sort_path(
                    NAME,
                    "O(N)",
                    false,
                    &[
                        "quick selects",
                        const_format::concatcp!($strat_name, " (dual pivot)"),
                        $dps_name,
                    ],
                );
            }

            #[cfg(test)]
            mod qs_test {
                #[test]
                fn correctness() {
                    crate::bench_registry::test_helpers::check_sort_subprocess_assert(
                        &super::ENTRY,
                        crate::bench_registry::test_helpers::DEFAULT_TIMEOUT,
                    );
                }
            }
        }
    };
}

// ── Single-pivot cross-product ───────────────────────────────────────────────
//
// Recursive × {Lomuto, Hoare, ThreeWay, Block} × pivots
register_quick_select_single!(rec_lomuto_first,    RecursiveQuickSelect, "recursive", Lomuto,   FirstElement);
register_quick_select_single!(rec_lomuto_middle,   RecursiveQuickSelect, "recursive", Lomuto,   MiddleElement);
register_quick_select_single!(rec_lomuto_last,     RecursiveQuickSelect, "recursive", Lomuto,   LastElement);
register_quick_select_single!(rec_lomuto_med3,     RecursiveQuickSelect, "recursive", Lomuto,   MedianOfThree);
register_quick_select_single!(rec_lomuto_ninther,  RecursiveQuickSelect, "recursive", Lomuto,   Ninther);

register_quick_select_single!(rec_hoare_first,     RecursiveQuickSelect, "recursive", Hoare,    FirstElement);
register_quick_select_single!(rec_hoare_middle,    RecursiveQuickSelect, "recursive", Hoare,    MiddleElement);
register_quick_select_single!(rec_hoare_last,      RecursiveQuickSelect, "recursive", Hoare,    LastElement);
register_quick_select_single!(rec_hoare_med3,      RecursiveQuickSelect, "recursive", Hoare,    MedianOfThree);
register_quick_select_single!(rec_hoare_ninther,   RecursiveQuickSelect, "recursive", Hoare,    Ninther);

register_quick_select_single!(rec_3way_first,      RecursiveQuickSelect, "recursive", ThreeWay, FirstElement);
register_quick_select_single!(rec_3way_middle,     RecursiveQuickSelect, "recursive", ThreeWay, MiddleElement);
register_quick_select_single!(rec_3way_last,       RecursiveQuickSelect, "recursive", ThreeWay, LastElement);
register_quick_select_single!(rec_3way_med3,       RecursiveQuickSelect, "recursive", ThreeWay, MedianOfThree);
register_quick_select_single!(rec_3way_ninther,    RecursiveQuickSelect, "recursive", ThreeWay, Ninther);

register_quick_select_single!(rec_block_first,     RecursiveQuickSelect, "recursive", Block,    FirstElement);
register_quick_select_single!(rec_block_middle,    RecursiveQuickSelect, "recursive", Block,    MiddleElement);
register_quick_select_single!(rec_block_last,      RecursiveQuickSelect, "recursive", Block,    LastElement);
register_quick_select_single!(rec_block_med3,      RecursiveQuickSelect, "recursive", Block,    MedianOfThree);
register_quick_select_single!(rec_block_ninther,   RecursiveQuickSelect, "recursive", Block,    Ninther);

// Iterative × {Lomuto, Hoare, ThreeWay, Block} × pivots
register_quick_select_single!(it_lomuto_first,     IterativeQuickSelect, "iterative", Lomuto,   FirstElement);
register_quick_select_single!(it_lomuto_middle,    IterativeQuickSelect, "iterative", Lomuto,   MiddleElement);
register_quick_select_single!(it_lomuto_last,      IterativeQuickSelect, "iterative", Lomuto,   LastElement);
register_quick_select_single!(it_lomuto_med3,      IterativeQuickSelect, "iterative", Lomuto,   MedianOfThree);
register_quick_select_single!(it_lomuto_ninther,   IterativeQuickSelect, "iterative", Lomuto,   Ninther);

register_quick_select_single!(it_hoare_first,      IterativeQuickSelect, "iterative", Hoare,    FirstElement);
register_quick_select_single!(it_hoare_middle,     IterativeQuickSelect, "iterative", Hoare,    MiddleElement);
register_quick_select_single!(it_hoare_last,       IterativeQuickSelect, "iterative", Hoare,    LastElement);
register_quick_select_single!(it_hoare_med3,       IterativeQuickSelect, "iterative", Hoare,    MedianOfThree);
register_quick_select_single!(it_hoare_ninther,    IterativeQuickSelect, "iterative", Hoare,    Ninther);

register_quick_select_single!(it_3way_first,       IterativeQuickSelect, "iterative", ThreeWay, FirstElement);
register_quick_select_single!(it_3way_middle,      IterativeQuickSelect, "iterative", ThreeWay, MiddleElement);
register_quick_select_single!(it_3way_last,        IterativeQuickSelect, "iterative", ThreeWay, LastElement);
register_quick_select_single!(it_3way_med3,        IterativeQuickSelect, "iterative", ThreeWay, MedianOfThree);
register_quick_select_single!(it_3way_ninther,     IterativeQuickSelect, "iterative", ThreeWay, Ninther);

register_quick_select_single!(it_block_first,      IterativeQuickSelect, "iterative", Block,    FirstElement);
register_quick_select_single!(it_block_middle,     IterativeQuickSelect, "iterative", Block,    MiddleElement);
register_quick_select_single!(it_block_last,       IterativeQuickSelect, "iterative", Block,    LastElement);
register_quick_select_single!(it_block_med3,       IterativeQuickSelect, "iterative", Block,    MedianOfThree);
register_quick_select_single!(it_block_ninther,    IterativeQuickSelect, "iterative", Block,    Ninther);

// ── Dual-pivot cross-product ─────────────────────────────────────────────────

type FirstFirst   = CombinedSelector<FirstElement, FirstElement>;
type MiddleMiddle = CombinedSelector<MiddleElement, MiddleElement>;
type FirstLast    = CombinedSelector<FirstElement, LastElement>;

// Variant labels mirror the dual-pivot quick *sort* registration so the
// same selector is named the same thing in both menus
// (see `dual_pivot_quick_sort.rs`'s `DPS` cross-product).
register_quick_select_dual!(dp_rec_first_first,   RecursiveDualPivotQuickSelect, "recursive", FirstFirst,       "first / first");
register_quick_select_dual!(dp_rec_mid_mid,       RecursiveDualPivotQuickSelect, "recursive", MiddleMiddle,     "middle / middle");
register_quick_select_dual!(dp_rec_first_last,    RecursiveDualPivotQuickSelect, "recursive", FirstLast,        "first / last");
register_quick_select_dual!(dp_rec_ninther,       RecursiveDualPivotQuickSelect, "recursive", NintherDualPivot, "ninther 1/3 + 2/3");

register_quick_select_dual!(dp_it_first_first,    IterativeDualPivotQuickSelect, "iterative", FirstFirst,       "first / first");
register_quick_select_dual!(dp_it_mid_mid,        IterativeDualPivotQuickSelect, "iterative", MiddleMiddle,     "middle / middle");
register_quick_select_dual!(dp_it_first_last,     IterativeDualPivotQuickSelect, "iterative", FirstLast,        "first / last");
register_quick_select_dual!(dp_it_ninther,        IterativeDualPivotQuickSelect, "iterative", NintherDualPivot, "ninther 1/3 + 2/3");
