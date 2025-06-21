use crate::create_sort;
use crate::traits::sort_traits::SortAlgo;

create_sort!(
    sort,
    "quick sort left right pivot optimised",
    "O(N Log(N))",
    false
);

fn partition<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> usize {
    let pivot = arr[0];
    let mut left = 1;
    let mut right = arr.len() - 1;
    while left <= right {
        while left <= right && logger.cmp_le_data(arr, left, pivot) {
            left += 1;
        }
        while left <= right && logger.cmp_gt_data(arr, right, pivot) {
            right -= 1;
        }
        if left < right {
            logger.swap(arr, left, right);
            left += 1;
            right -= 1;
        }
    }
    logger.swap(arr, 0, right);
    right
}

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 32 {
        type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
        SmallSort::sort(arr, logger);
        return;
    }
    let partition_index = partition(arr, logger);
    sort(&mut arr[..partition_index], logger);
    sort(&mut arr[partition_index + 1..], logger);
}
