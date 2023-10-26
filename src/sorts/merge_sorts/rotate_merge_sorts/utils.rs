use crate::rotations::rotate::rotate;
use crate::traits::log_traits::SortLogger;

pub fn binary_search<T: Ord + Copy, U: SortLogger<T>>(
    arr: &[T],
    target: T,
    logger: &mut U,
) -> usize {
    let mut top = arr.len();
    let mut bot = 0;
    let mut mid = (top + bot) / 2;
    while bot < top {
        mid = (top + bot) / 2;
        if logger.cmp_lt_data(arr, mid, target) {
            bot = mid + 1;
        } else {
            top = mid;
        }
    }
    if mid == arr.len() {
        return mid;
    }
    if logger.cmp_lt_data(arr, mid, target) {
        mid += 1
    }
    mid
}

pub fn rotate_merge<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], middle: usize, logger: &mut U) {
    if middle == 0 || middle == arr.len() {
        return;
    }
    let mid_left = middle / 2;
    let mid_ind = binary_search(&arr[middle..], arr[mid_left], logger);

    let mid_left_new = middle + mid_ind - mid_left;
    rotate(
        &mut arr[mid_left..middle + mid_ind],
        middle - mid_left,
        logger,
    );
    rotate_merge(&mut arr[..mid_left_new], mid_left, logger);
    rotate_merge(
        &mut arr[mid_left_new..],
        middle + mid_ind - mid_left_new,
        logger,
    );
}
