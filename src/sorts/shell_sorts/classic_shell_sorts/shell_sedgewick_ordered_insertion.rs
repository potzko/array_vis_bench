use crate::create_sort;

create_sort!(
    sort,
    "shell sort sedgewick ordered insertion",
    "O(N^(4/3))",
    false
);

#[inline]
fn sequence_sedgewick(iter: usize) -> usize {
    if iter == 0 {
        return 1;
    }
    4_usize.pow(iter as u32) + 3 * 2_usize.pow(iter as u32 - 1) + 1
}

fn vec_sedgewick(len: usize) -> Vec<usize> {
    let mut ret = Vec::new();
    let mut iter = 0;
    loop {
        let num = sequence_sedgewick(iter);
        if num >= len {
            break;
        }
        ret.push(num);
        iter += 1
    }
    ret
}

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let len = arr.len();
    let jumps = vec_sedgewick(len);
    for &jump in jumps.iter().rev() {
        logger.mark_mssg(&format!("jump = {}", jump));
        for i in 0..jump {
            insertion_sort_jump(arr, i, arr.len(), jump, logger)
        }
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
        let mut ind = i;
        while ind != start {
            if logger.cond_swap_lt(arr, ind, ind - jump) {
                ind -= jump;
            } else {
                break;
            }
        }
    }
}
