use std::marker::PhantomData;

use crate::utils::small_sort::{insertion_sort, DeferredSmallSort};
use crate::traits::log_traits::SortLogger;

use super::partitions::PartitionScheme;
use super::pivot_selectors::PivotSelector;

combo_codegen::sort_family!(
    type = DeferredQuickSort<{P}, {V}, {DSS}>,
    uses = [
        "crate::utils::small_sort::DeferredInsertion",
        "super::partitions::{Block, Hoare, Lomuto, MovingPivot, ThreeWay}",
        "super::pivot_selectors::{FirstElement, LastElement, MedianOfMedians, MedianOfThree, MiddleElement, Ninther, CombinedSelector, NintherDualPivot}",
        "super::deferred_quick_sort::DeferredQuickSort",
    ],
    P: Partition,
    V: PivotSelector,
    DSS: DeferredSmallSort,
    name = "quick sort classic deferred",
    big_o = "O(N Log(N))",
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
        insertion_sort(arr, logger);
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
