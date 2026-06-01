//! Standalone-partition registration: each (PartitionScheme × PivotSelector)
//! pair is registered as a `Category::Partition` algorithm that takes the
//! same `SortInput` shape as a sort. The pivot selector lives inside the
//! wrapper so the public partition fn is just `(&mut [usize], &mut Logger)`.
//!
//! 5 partition schemes × 6 pivot selectors = 30 standalone partitions.
//!
//! This crate has no public API beyond [`LINK_ANCHOR`] — its job is the
//! `#[ctor]` + `#[linkme::distributed_slice]` side-effects that fire
//! when it's linked. Downstream wiring crates reference [`LINK_ANCHOR`]
//! from a `#[used]` static so the linker doesn't drop the object file
//! under `--gc-sections` (which kills both the ctor and the
//! distributed-slice entries with it).

/// Force-link anchor — see module docs.
pub static LINK_ANCHOR: () = ();

use partition_block::Block;
use partition_hoare::LeftRightPartition;
use partition_lomuto::LeftLeftPartition;
use partition_moving_pivot::MovingPivot;
use partition_three_way::ThreeWay;
use pivot_first::FirstElement;
use pivot_last::LastElement;
use pivot_median3::MedianOfThree;
use pivot_median_of_medians::MedianOfMedians;
use pivot_middle::MiddleElement;
use pivot_ninther::Ninther;

/// Wrap a `(PartitionScheme, PivotSelector)` pair as a standalone
/// algorithm. Each invocation lives in its own private inner module so
/// the per-leaf helper names don't collide.
macro_rules! register_partition {
    ($mod:ident, $part:ty, $piv:ty) => {
        mod $mod {
            use super::*;
            use array_vis_bench_traits::{
                with_partition_scratch, PartitionScheme, PartitionVisitor, PivotSelector,
            };
            use sort_logger::{NoOpLogger, SortLogger};
            use std::ops::Range;

            const NAME: &str = const_format::concatcp!(
                "partition: ",
                <$part as PartitionScheme>::NAME,
                "<",
                <$piv as PivotSelector>::NAME,
                ">",
            );

            /// Single-pivot visitor that just remembers the bounds of
            /// the gap between the two unsorted regions (== where the
            /// placed/sorted run sits).
            struct BoundsVisitor { left_end: usize, right_start: usize, n: u8 }
            impl BoundsVisitor {
                fn new(len: usize) -> Self { Self { left_end: 0, right_start: len, n: 0 } }
            }
            impl PartitionVisitor for BoundsVisitor {
                #[inline(always)]
                fn unsorted(&mut self, r: Range<usize>) {
                    if self.n == 0 {
                        self.left_end = r.end;
                    } else if self.n == 1 {
                        self.right_start = r.start;
                    }
                    self.n += 1;
                }
            }

            /// dyn-logger entry — drops the bounds because the
            /// visualiser only cares about the event stream.
            fn partition_dyn(
                arr: &mut [usize],
                logger: &mut dyn SortLogger<usize>,
            ) {
                if arr.len() < 2 {
                    return;
                }
                let pivot = <$piv as PivotSelector>::select(arr, logger);
                let mut v = BoundsVisitor::new(arr.len());
                with_partition_scratch::<$part, usize, _, _>(logger, |logger, scratch| {
                    <$part as PartitionScheme>::partition(arr, logger, &[pivot], scratch, &mut v);
                });
            }

            /// NoOp-logger entry — keeps the bounds so the battery can
            /// verify `max(arr[..left_end]) ≤ min(arr[right_start..])`.
            fn partition_noop(
                arr: &mut [usize],
                logger: &mut NoOpLogger,
            ) -> (usize, usize) {
                if arr.len() < 2 {
                    return (0, arr.len());
                }
                let pivot = <$piv as PivotSelector>::select(arr, logger);
                let mut v = BoundsVisitor::new(arr.len());
                with_partition_scratch::<$part, usize, _, _>(logger, |logger, scratch| {
                    <$part as PartitionScheme>::partition(arr, logger, &[pivot], scratch, &mut v);
                });
                (v.left_end, v.right_start)
            }

            fn run_with_input(
                input_name: &str,
                config: &array_vis_bench_core::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                array_vis_bench_core::bench_registry::run_partition_with_input(
                    input_name, config, partition_dyn, logger,
                );
            }
            fn run_correctness() {
                array_vis_bench_core::bench_registry::correctness::partition_battery(
                    partition_noop, NAME,
                );
            }

            // One partition step = partition scan + pivot selection. Both
            // axes declare their own `HasTimeBounds` impls; sum picks the
            // dominant cost (Block / LeftRightPartition / LeftLeftPartition / ThreeWay / MovingPivot
            // are all O(N), MedianOfMedians is O(N), the rest are O(1)).
            #[linkme::distributed_slice(array_vis_bench_core::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: array_vis_bench_core::bench_registry::AlgorithmEntry =
                array_vis_bench_core::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: array_vis_bench_core::bench_registry::Category::Partition,
                    worst: array_vis_bench_traits::Complexity::sum(
                        <$part as array_vis_bench_traits::composable::HasTimeBounds>::WORST,
                        <$piv as array_vis_bench_traits::composable::HasTimeBounds>::WORST,
                    ),
                    best: array_vis_bench_traits::Complexity::sum(
                        <$part as array_vis_bench_traits::composable::HasTimeBounds>::BEST,
                        <$piv as array_vis_bench_traits::composable::HasTimeBounds>::BEST,
                    ),
                    average: array_vis_bench_traits::Complexity::sum(
                        <$part as array_vis_bench_traits::composable::HasTimeBounds>::AVERAGE,
                        <$piv as array_vis_bench_traits::composable::HasTimeBounds>::AVERAGE,
                    ),
                    space: array_vis_bench_traits::Complexity::sum(
                        <$part as array_vis_bench_traits::composable::HasSpace>::SPACE,
                        <$piv as array_vis_bench_traits::composable::HasSpace>::SPACE,
                    ),
                    stable: <$part as array_vis_bench_traits::composable::HasStability>::STABLE
                        && <$piv as array_vis_bench_traits::composable::HasStability>::STABLE,
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

            }
    };
}

// ── LeftLeftPartition × pivots ──────────────────────────────────────────────────────────
register_partition!(lomuto_first,    LeftLeftPartition, FirstElement);
register_partition!(lomuto_middle,   LeftLeftPartition, MiddleElement);
register_partition!(lomuto_last,     LeftLeftPartition, LastElement);
register_partition!(lomuto_med3,     LeftLeftPartition, MedianOfThree);
register_partition!(lomuto_medmeds,  LeftLeftPartition, MedianOfMedians);
register_partition!(lomuto_ninther,  LeftLeftPartition, Ninther);

// ── LeftRightPartition × pivots ───────────────────────────────────────────────────────────
register_partition!(hoare_first,     LeftRightPartition, FirstElement);
register_partition!(hoare_middle,    LeftRightPartition, MiddleElement);
register_partition!(hoare_last,      LeftRightPartition, LastElement);
register_partition!(hoare_med3,      LeftRightPartition, MedianOfThree);
register_partition!(hoare_medmeds,   LeftRightPartition, MedianOfMedians);
register_partition!(hoare_ninther,   LeftRightPartition, Ninther);

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
