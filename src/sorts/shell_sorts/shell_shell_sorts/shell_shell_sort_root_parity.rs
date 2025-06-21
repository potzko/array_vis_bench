use crate::create_sort;

create_sort!(sort, "shell shell sort root parity", "O(N^2)", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    sort_rec(arr, 1, logger);
}

fn sort_rec<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    let len = arr.len() / jump;
    if len <= 4 {
        insertion_sort_jump(arr, jump, logger);
        return;
    }
    let num = (len as f64).sqrt() as usize;
    for i in 0..num {
        sort_rec(&mut arr[jump * i..], jump * num, logger);
    }
    let num = num - 1;
    if len >= num * 16 {
        for i in 0..num {
            insertion_sort_jump(&mut arr[jump * i..], jump * num, logger);
        }
    }
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
