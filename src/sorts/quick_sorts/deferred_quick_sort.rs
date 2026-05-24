use std::marker::PhantomData;

use crate::utils::small_sort::DeferredSmallSort;
use crate::traits::log_traits::SortLogger;

use super::partitions::PartitionScheme;
use super::pivot_selectors::PivotSelector;

combo_codegen::family!(
    type = DeferredQuickSort<{P}, {V}, {DSS}>,
    uses = [
        "crate::utils::small_sort::DeferredInsertion",
        "crate::utils::small_sort::{LinearInsertion, BinaryInsertion}",
        "super::partitions::{Block, Hoare, Lomuto, MovingPivot, ThreeWay}",
        "super::pivot_selectors::{FirstElement, LastElement, MedianOfMedians, MedianOfThree, MiddleElement, Ninther, CombinedSelector, NintherDualPivot}",
        "super::deferred_quick_sort::DeferredQuickSort",
    ],
    P: Partition,
    V: PivotSelector,
    DSS: DeferredSmallSort,
    name = "quick sort classic deferred",
    big_o = inherited,
    stable = false,
    direct_sort = true,
    path = ["quick sorts", "classic deferred", "{P}", "{V}", "{DSS}"],
);

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
impl<P, V, DSS> crate::traits::composable::HasTimeBounds for DeferredQuickSort<P, V, DSS>
where
    P: super::partitions::PartitionScheme + crate::traits::composable::HasTimeBounds,
    V: super::pivot_selectors::PivotSelector
        + crate::traits::composable::HasTimeBounds
        + crate::traits::composable::PivotQuality,
    DSS: crate::utils::small_sort::DeferredSmallSort,
{
    const WORST: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::product(
        if V::DEGENERATES {
            crate::traits::complexity::Complexity::N1
        } else {
            crate::traits::complexity::Complexity::LOG_N
        },
        crate::traits::complexity::Complexity::sum(P::WORST, V::WORST),
    );
    const BEST: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::product(
        crate::traits::complexity::Complexity::LOG_N,
        crate::traits::complexity::Complexity::sum(P::BEST, V::BEST),
    );
    const AVERAGE: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::product(
        crate::traits::complexity::Complexity::LOG_N,
        crate::traits::complexity::Complexity::sum(P::AVERAGE, V::AVERAGE),
    );
}
impl<P, V, DSS> crate::traits::composable::HasSpace for DeferredQuickSort<P, V, DSS>
where
    P: super::partitions::PartitionScheme + crate::traits::composable::HasSpace,
    V: super::pivot_selectors::PivotSelector + crate::traits::composable::HasSpace,
    DSS: crate::utils::small_sort::DeferredSmallSort,
{
    const SPACE: crate::traits::complexity::Complexity = crate::traits::complexity::Complexity::sum(
        crate::traits::complexity::Complexity::LOG_N,
        crate::traits::complexity::Complexity::sum(P::SPACE, V::SPACE),
    );
}
impl<P, V, DSS> crate::traits::composable::HasStability for DeferredQuickSort<P, V, DSS>
where
    P: super::partitions::PartitionScheme + crate::traits::composable::HasStability,
    V: super::pivot_selectors::PivotSelector + crate::traits::composable::HasStability,
    DSS: crate::utils::small_sort::DeferredSmallSort,
{
    const STABLE: bool = P::STABLE && V::STABLE;
}
