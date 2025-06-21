use crate::create_sort;

create_sort!(
    sort,
    "circle sort recursive increasing size",
    "O(N^log^2(N))",
    false
);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    while circle_sort_rec(arr, 0, arr.len() - 1, logger) {}
}

fn circle_sort_rec<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> bool {
    let mut swapped = false;

    if start == end {
        return false;
    }

    let (mut start_tmp, mut end_tmp) = (start, end);

    while start_tmp < end_tmp {
        if logger.cond_swap_lt(arr, end_tmp, start_tmp) {
            swapped = true;
        }
        start_tmp += 1;
        end_tmp -= 1;
    }

    if start_tmp == end_tmp && logger.cond_swap_lt(arr, end_tmp + 1, start_tmp) {
        swapped = true;
    }

    let mid = start + (end - start) / 2;
    let first_half = circle_sort_rec(arr, start, mid, logger);
    let second_half = circle_sort_rec(arr, mid + 1, end, logger);

    swapped || first_half || second_half
}
