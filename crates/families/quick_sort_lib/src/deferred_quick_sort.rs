use std::marker::PhantomData;
use std::ops::Range;

use array_vis_bench_traits::composable::PivotQuality;
use array_vis_bench_traits::DeferredSmallSort;
use sort_logger::SortLogger;

use array_vis_bench_traits::{with_partition_scratch, PartitionScheme, PartitionVisitor};

struct DeferredVisitor {
    ranges: [Range<usize>; 4],
    n: u8,
}

impl PartitionVisitor for DeferredVisitor {
    #[inline(always)]
    fn unsorted(&mut self, r: Range<usize>) {
        unsafe { *self.ranges.get_unchecked_mut(self.n as usize) = r };
        self.n += 1;
    }
}

fn deferred_recursive<
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionScheme,
    DSS: DeferredSmallSort,
>(
    arr: &mut [T],
    scratch: &mut [usize],
    logger: &mut U,
) {
    if arr.len() < 2 {
        return;
    }
    if arr.len() <= DSS::THRESHOLD {
        return;
    }
    // Self-pivoting partition: the driver supplies no pivots.
    let mut visitor = DeferredVisitor { ranges: [0..0, 0..0, 0..0, 0..0], n: 0 };
    P::partition::<T, U, _>(arr, logger, &[], scratch, &mut visitor);
    let n = visitor.n as usize;
    let mut i = 0;
    while i < n {
        let r = visitor.ranges[i].clone();
        deferred_recursive::<T, U, P, DSS>(&mut arr[r], scratch, logger);
        i += 1;
    }
}

/// The **canonical** deferred quicksort driver — the deferred counterpart of
/// [`CanonicalQuickSort`](crate::quick_sort::CanonicalQuickSort): the partition
/// self-splits, the sort carries no pivot parameter, and `DSS::final_pass`
/// cleans up the deferred leaves at the end.
pub struct DeferredCanonicalQuickSort<P: PartitionScheme, DSS: DeferredSmallSort>(
    PhantomData<(P, DSS)>,
);

impl<P: PartitionScheme, DSS: DeferredSmallSort> DeferredCanonicalQuickSort<P, DSS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        // The single deferred quicksort recursion.
        with_partition_scratch::<P, T, U, _>(logger, |logger, scratch| {
            deferred_recursive::<T, U, P, DSS>(arr, scratch, logger);
        });
        DSS::final_pass(arr, logger);
    }
}

array_vis_bench_traits::direct_sort!(DeferredCanonicalQuickSort<P, DSS>; P: PartitionScheme, DSS: DeferredSmallSort);

// ── Composable annotations for the deferred pivotless driver ──────────
// Depth is O(N) if the partition can degenerate, else O(log N) — read off
// the partition `P` (pivot-agnostic, like the in-place pivotless driver).
impl<P, DSS> array_vis_bench_traits::composable::HasTimeBounds
    for DeferredCanonicalQuickSort<P, DSS>
where
    P: array_vis_bench_traits::PartitionScheme
        + array_vis_bench_traits::composable::HasTimeBounds
        + PivotQuality,
    DSS: array_vis_bench_traits::DeferredSmallSort,
{
    const WORST: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::product(
            if P::DEGENERATES {
                array_vis_bench_traits::Complexity::N1
            } else {
                array_vis_bench_traits::Complexity::LOG_N
            },
            P::WORST,
        );
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::product(
        array_vis_bench_traits::Complexity::LOG_N,
        P::BEST,
    );
    const AVERAGE: array_vis_bench_traits::Complexity =
        array_vis_bench_traits::Complexity::product(
            array_vis_bench_traits::Complexity::LOG_N,
            P::AVERAGE,
        );
}
impl<P, DSS> array_vis_bench_traits::composable::HasSpace for DeferredCanonicalQuickSort<P, DSS>
where
    P: array_vis_bench_traits::PartitionScheme + array_vis_bench_traits::composable::HasSpace,
    DSS: array_vis_bench_traits::DeferredSmallSort,
{
    const SPACE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::sum(
        array_vis_bench_traits::Complexity::LOG_N,
        P::SPACE,
    );
}
impl<P, DSS> array_vis_bench_traits::composable::HasStability
    for DeferredCanonicalQuickSort<P, DSS>
where
    P: array_vis_bench_traits::PartitionScheme + array_vis_bench_traits::composable::HasStability,
    DSS: array_vis_bench_traits::DeferredSmallSort,
{
    const STABLE: bool = P::STABLE;
}
