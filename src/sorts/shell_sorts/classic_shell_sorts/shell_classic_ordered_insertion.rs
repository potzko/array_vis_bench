use crate::create_sort;

create_sort!(
    sort,
    "shell sort classic ordered insertion",
    "O(N^2)",
    false
);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut jump = arr.len() / 2;
    while jump >= 1 {
        logger.mark(format!("jump = {}", jump));
        for i in 0..jump {
            insertion_sort_jump(arr, i, arr.len(), jump, logger);
        }
        jump /= 2;
    }
}

fn insertion_sort_jump<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    jump: usize,
    logger: &mut U,
) {
    for i in (start..end).step_by(jump) {
        let mut ind: usize = i;
        while ind != start {
            if logger.cond_swap_lt(arr, ind, ind - jump) {
                ind -= jump;
            } else {
                break;
            }
        }
    }
}
