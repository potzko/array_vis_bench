use crate::create_sort;

create_sort!(sort, "shell shell sort 3 parity", "O(N^2)", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    sort_rec(arr, 1, logger);
}

fn sort_rec<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    let len = arr.len() / jump;

    if len < 2 {
        return;
    }
    sort_rec(arr, jump * 3, logger);
    sort_rec(&mut arr[jump..], jump * 3, logger);
    sort_rec(&mut arr[jump * 2..], jump * 3, logger);
    insertion_sort_jump(arr, jump, logger);
}

fn insertion_sort_jump<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    for i in (0..arr.len()).step_by(jump) {
        let mut ind = i;
        while ind != 0 {
            if logger.cond_swap_le(arr, ind, ind - jump) {
                ind -= jump;
            } else {
                break;
            }
        }
    }
}
