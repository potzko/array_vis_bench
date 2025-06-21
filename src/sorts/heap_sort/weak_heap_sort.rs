use crate::create_sort;

create_sort!(sort, "weak heap sort", "O(N*log(N))", false);

pub fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) {
    let n = arr.len();

    if n < 2 {
        return;
    }

    let mut bottom_skips = vec![0; (n + 7) / 8];
    for i in (1..n).rev() {
        let mut ii = i;
        while ii & 1 == get_bitwise_flag(&bottom_skips, ii >> 1) {
            ii >>= 1;
        }
        let big_parent = ii >> 1;
        weak_heap_merge(arr, &mut bottom_skips, big_parent, i, logger);
    }

    for i in (2..n).rev() {
        logger.swap(arr, 0, i);
        let mut current = 1;
        let mut next = 2 * current + get_bitwise_flag(&bottom_skips, current);
        while next < i {
            current = next;
            next = 2 * current + get_bitwise_flag(&bottom_skips, current);
        }
        while current > 0 {
            weak_heap_merge(arr, &mut bottom_skips, 0, current, logger);
            current >>= 1;
        }
    }
    arr.swap(0, 1);
}

fn weak_heap_merge<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    bottom_skips: &mut [u8],
    ind_a: usize,
    ind_b: usize,
    logger: &mut U,
) {
    if logger.cond_swap_lt(arr, ind_a, ind_b) {
        toggle_bitwise_flag(bottom_skips, ind_b);
    }
}

fn get_bitwise_flag(bottom_skips: &[u8], ind: usize) -> usize {
    ((bottom_skips[ind >> 3] >> (ind & 7)) & 1).into()
}

fn toggle_bitwise_flag(bottom_skips: &mut [u8], ind: usize) {
    bottom_skips[ind >> 3] ^= 1 << (ind & 7)
}
