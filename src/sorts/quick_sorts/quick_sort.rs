use std::marker::PhantomData;

use crate::utils::small_sort::SmallSort;
use crate::traits::log_traits::SortLogger;

use super::partitions::PartitionScheme;
use super::pivot_selectors::PivotSelector;

pub struct QuickSort<P: PartitionScheme, V: PivotSelector, SS: SmallSort>(
    PhantomData<(P, V, SS)>,
);

impl<P: PartitionScheme, V: PivotSelector, SS: SmallSort> QuickSort<P, V, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        quick_sort_recursive::<T, U, P, V, SS>(arr, logger);
    }
}

fn quick_sort_recursive<
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    P: PartitionScheme,
    V: PivotSelector,
    SS: SmallSort,
>(
    arr: &mut [T],
    logger: &mut U,
) {
    if SS::THRESHOLD > 0 && arr.len() <= SS::THRESHOLD {
        SS::sort(arr, logger);
        return;
    }
    if arr.len() < 2 {
        return;
    }
    let pivot_idx = V::select(arr, logger);
    let (left_end, right_start) = P::partition(arr, logger, pivot_idx);
    quick_sort_recursive::<T, U, P, V, SS>(&mut arr[..left_end], logger);
    quick_sort_recursive::<T, U, P, V, SS>(&mut arr[right_start..], logger);
}
