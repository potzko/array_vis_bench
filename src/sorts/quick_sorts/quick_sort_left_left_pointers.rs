use crate::create_sort;
use crate::traits::sort_traits::SortAlgo;

create_sort!(sort, "quick sort left left pointers", "O(N Log(N))", false);

fn partition<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> usize {
    assert!(arr.len() >= 2);
    let pivot = arr[arr.len() - 1];
    let mut small = 0;
    for i in 0..arr.len() - 1 {
        if logger.cmp_le_data(arr, i, pivot) {
            logger.swap(arr, i, small);
            small += 1;
        }
    }
    logger.swap(arr, small, arr.len() - 1);
    small
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
