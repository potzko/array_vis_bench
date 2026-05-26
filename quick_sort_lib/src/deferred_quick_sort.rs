use std::marker::PhantomData;

use array_vis_bench_traits::DeferredSmallSort;
use sort_logger::SortLogger;

use array_vis_bench_traits::PartitionScheme;
use array_vis_bench_traits::PivotSelector;

pub struct DeferredQuickSort<P: PartitionScheme, V: PivotSelector, DSS: DeferredSmallSort>(
    PhantomData<(P, V, DSS)>,
);

impl<P: PartitionScheme, V: PivotSelector, DSS: DeferredSmallSort> DeferredQuickSort<P, V, DSS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        deferred_recursive::<T, U, P, V, DSS>(arr, logger);
        DSS::final_pass(arr, logger);
    }
}

fn deferred_recursive<
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionScheme,
    V: PivotSelector,
    DSS: DeferredSmallSort,
>(
    arr: &mut [T],
    logger: &mut U,
) {
    if arr.len() < 2 {
        return;
    }
    if arr.len() <= DSS::THRESHOLD {
        return;
    }
    let pivot_idx = V::select(arr, logger);
    let (left_end, right_start) = P::partition(arr, logger, pivot_idx);
    deferred_recursive::<T, U, P, V, DSS>(&mut arr[..left_end], logger);
    deferred_recursive::<T, U, P, V, DSS>(&mut arr[right_start..], logger);
}

// Same composition profile as QuickSort: worst depends on pivot quality,
// best/average is O(N log N). DeferredSmallSort runs on bounded leaves
// during a single final pass, so its contribution is O(1) at composition
// time (bounded by SS::THRESHOLD).
impl<P, V, DSS> array_vis_bench_traits::composable::HasTimeBounds for DeferredQuickSort<P, V, DSS>
where
    P: array_vis_bench_traits::PartitionScheme + array_vis_bench_traits::composable::HasTimeBounds,
    V: array_vis_bench_traits::PivotSelector
        + array_vis_bench_traits::composable::HasTimeBounds
        + array_vis_bench_traits::composable::PivotQuality,
    DSS: array_vis_bench_traits::DeferredSmallSort,
{
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::product(
        if V::DEGENERATES {
            array_vis_bench_traits::Complexity::N1
        } else {
            array_vis_bench_traits::Complexity::LOG_N
        },
        array_vis_bench_traits::Complexity::sum(P::WORST, V::WORST),
    );
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::product(
        array_vis_bench_traits::Complexity::LOG_N,
        array_vis_bench_traits::Complexity::sum(P::BEST, V::BEST),
    );
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::product(
        array_vis_bench_traits::Complexity::LOG_N,
        array_vis_bench_traits::Complexity::sum(P::AVERAGE, V::AVERAGE),
    );
}
impl<P, V, DSS> array_vis_bench_traits::composable::HasSpace for DeferredQuickSort<P, V, DSS>
where
    P: array_vis_bench_traits::PartitionScheme + array_vis_bench_traits::composable::HasSpace,
    V: array_vis_bench_traits::PivotSelector + array_vis_bench_traits::composable::HasSpace,
    DSS: array_vis_bench_traits::DeferredSmallSort,
{
    const SPACE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::sum(
        array_vis_bench_traits::Complexity::LOG_N,
        array_vis_bench_traits::Complexity::sum(P::SPACE, V::SPACE),
    );
}
impl<P, V, DSS> array_vis_bench_traits::composable::HasStability for DeferredQuickSort<P, V, DSS>
where
    P: array_vis_bench_traits::PartitionScheme + array_vis_bench_traits::composable::HasStability,
    V: array_vis_bench_traits::PivotSelector + array_vis_bench_traits::composable::HasStability,
    DSS: array_vis_bench_traits::DeferredSmallSort,
{
    const STABLE: bool = P::STABLE && V::STABLE;
}
