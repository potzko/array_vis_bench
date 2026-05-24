//! Standalone-partition registration: each (PartitionScheme × PivotSelector)
//! pair is registered as a `Category::Partition` algorithm that takes the
//! same `SortInput` shape as a sort. The pivot selector lives inside the
//! wrapper so the public partition fn is just `(&mut [usize], &mut Logger)`.
//!
//! 5 partition schemes × 6 pivot selectors = 30 standalone partitions.

use crate::sorts::quick_sorts::partitions::{Block, Hoare, Lomuto, MovingPivot, ThreeWay};
use crate::sorts::quick_sorts::pivot_selectors::{
    FirstElement, LastElement, MedianOfMedians, MedianOfThree, MiddleElement, Ninther,
};

/// Wrap a `(PartitionScheme, PivotSelector)` pair as a standalone
/// algorithm. Each invocation lives in its own private inner module so
/// the per-leaf helper names don't collide.
macro_rules! register_partition {
    ($mod:ident, $part:ty, $piv:ty) => {
        mod $mod {
            use super::*;
            use crate::sorts::quick_sorts::partitions::PartitionScheme;
            use crate::sorts::quick_sorts::pivot_selectors::PivotSelector;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const NAME: &str = const_format::concatcp!(
                "partition: ",
                <$part as PartitionScheme>::NAME,
                "<",
                <$piv as PivotSelector>::NAME,
                ">",
            );

            /// dyn-logger entry — drops the (left_end, right_start)
            /// return because the visualiser only cares about the
            /// stream of events.
            fn partition_dyn(
                arr: &mut [usize],
                logger: &mut dyn SortLogger<usize>,
            ) {
                if arr.len() < 2 {
                    return;
                }
                let pivot = <$piv as PivotSelector>::select(arr, logger);
                let _ = <$part as PartitionScheme>::partition(arr, logger, pivot);
            }

            /// NoOp-logger entry — keeps the return so the battery can
            /// verify `max(arr[..left_end]) ≤ min(arr[right_start..])`.
            fn partition_noop(
                arr: &mut [usize],
                logger: &mut NoOpLogger,
            ) -> (usize, usize) {
                if arr.len() < 2 {
                    return (0, arr.len());
                }
                let pivot = <$piv as PivotSelector>::select(arr, logger);
                <$part as PartitionScheme>::partition(arr, logger, pivot)
            }

            fn run_with_input(
                input_name: &str,
                config: &crate::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                crate::bench_registry::run_partition_with_input(
                    input_name, config, partition_dyn, logger,
                );
            }
            fn run_correctness() {
                crate::bench_registry::correctness::partition_battery(
                    partition_noop, NAME,
                );
            }

            // One partition step = partition scan + pivot selection. Both
            // axes declare their own `HasTimeBounds` impls; sum picks the
            // dominant cost (Block / Hoare / Lomuto / ThreeWay / MovingPivot
            // are all O(N), MedianOfMedians is O(N), the rest are O(1)).
            #[linkme::distributed_slice(crate::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: crate::bench_registry::AlgorithmEntry =
                crate::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: crate::bench_registry::Category::Partition,
                    worst: crate::traits::complexity::Complexity::sum(
                        <$part as crate::traits::composable::HasTimeBounds>::WORST,
                        <$piv as crate::traits::composable::HasTimeBounds>::WORST,
                    ),
                    best: crate::traits::complexity::Complexity::sum(
                        <$part as crate::traits::composable::HasTimeBounds>::BEST,
                        <$piv as crate::traits::composable::HasTimeBounds>::BEST,
                    ),
                    average: crate::traits::complexity::Complexity::sum(
                        <$part as crate::traits::composable::HasTimeBounds>::AVERAGE,
                        <$piv as crate::traits::composable::HasTimeBounds>::AVERAGE,
                    ),
                    space: crate::traits::complexity::Complexity::sum(
                        <$part as crate::traits::composable::HasSpace>::SPACE,
                        <$piv as crate::traits::composable::HasSpace>::SPACE,
                    ),
                    stable: <$part as crate::traits::composable::HasStability>::STABLE
                        && <$piv as crate::traits::composable::HasStability>::STABLE,
                    adaptive: false,
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
                        "partitions",
                        <$part as PartitionScheme>::NAME,
                        <$piv as PivotSelector>::NAME,
                    ],
                );
            }

            #[cfg(test)]
            mod partition_test {
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

// ── Lomuto × pivots ──────────────────────────────────────────────────────────
register_partition!(lomuto_first,    Lomuto, FirstElement);
register_partition!(lomuto_middle,   Lomuto, MiddleElement);
register_partition!(lomuto_last,     Lomuto, LastElement);
register_partition!(lomuto_med3,     Lomuto, MedianOfThree);
register_partition!(lomuto_medmeds,  Lomuto, MedianOfMedians);
register_partition!(lomuto_ninther,  Lomuto, Ninther);

// ── Hoare × pivots ───────────────────────────────────────────────────────────
register_partition!(hoare_first,     Hoare, FirstElement);
register_partition!(hoare_middle,    Hoare, MiddleElement);
register_partition!(hoare_last,      Hoare, LastElement);
register_partition!(hoare_med3,      Hoare, MedianOfThree);
register_partition!(hoare_medmeds,   Hoare, MedianOfMedians);
register_partition!(hoare_ninther,   Hoare, Ninther);

// ── ThreeWay × pivots ────────────────────────────────────────────────────────
register_partition!(threeway_first,    ThreeWay, FirstElement);
register_partition!(threeway_middle,   ThreeWay, MiddleElement);
register_partition!(threeway_last,     ThreeWay, LastElement);
register_partition!(threeway_med3,     ThreeWay, MedianOfThree);
register_partition!(threeway_medmeds,  ThreeWay, MedianOfMedians);
register_partition!(threeway_ninther,  ThreeWay, Ninther);

// ── Block × pivots ───────────────────────────────────────────────────────────
register_partition!(block_first,     Block, FirstElement);
register_partition!(block_middle,    Block, MiddleElement);
register_partition!(block_last,      Block, LastElement);
register_partition!(block_med3,      Block, MedianOfThree);
register_partition!(block_medmeds,   Block, MedianOfMedians);
register_partition!(block_ninther,   Block, Ninther);

// ── MovingPivot × pivots ─────────────────────────────────────────────────────
register_partition!(moving_first,    MovingPivot, FirstElement);
register_partition!(moving_middle,   MovingPivot, MiddleElement);
register_partition!(moving_last,     MovingPivot, LastElement);
register_partition!(moving_med3,     MovingPivot, MedianOfThree);
register_partition!(moving_medmeds,  MovingPivot, MedianOfMedians);
register_partition!(moving_ninther,  MovingPivot, Ninther);
