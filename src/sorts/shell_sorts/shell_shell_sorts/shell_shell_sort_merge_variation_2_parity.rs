use crate::create_sort;

create_sort!(sort, "shell sort 2.25 shrink factor", "O(N^2)", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut tmp = logger.copy_aux_arr_t(arr);
    sort_rec(arr, &mut tmp, 1, logger);
}

fn sort_rec<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    target: &mut [T],
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    let len = arr.len() / jump;

    if len < 2 {
        return;
    }
    sort_rec(arr, target, jump * 2, logger);
    sort_rec(&mut arr[jump..], &mut target[jump..], jump * 2, logger);
    merge_jump(&target, arr, jump, logger);
}

pub fn jump_merge<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    target: &mut [T],
    arr: &[T],
    start_a: usize,
    jump_a: usize,
    start_b: usize,
    jump_b: usize,
    logger: &mut U,
) {
    let (mut start_a, mut start_b) = (start_a, start_b);
    let (mut tmp_start_a, mut tmp_start_b) = (start_a, start_b);
    let len = target.len();
    while start_a < len && start_b < len {
        let next_ind: usize = if tmp_start_a < tmp_start_b {
            tmp_start_a += jump_a;
            tmp_start_a - jump_a
        } else {
            tmp_start_b += jump_b;
            tmp_start_b - jump_b
        };
        if logger.cmp_lt(target, start_a, start_b) {
            logger.write_accross(arr, next_ind, target, start_a);
            start_a += jump_a;
        } else {
            logger.write_accross(arr, next_ind, target, start_b);
            start_b += jump_b
        };
    }
    for ind in (start_a..len).step_by(jump_a) {
        let next_ind: usize = if tmp_start_a < tmp_start_b {
            tmp_start_a += jump_a;
            tmp_start_a - jump_a
        } else {
            tmp_start_b += jump_b;
            tmp_start_b - jump_b
        };
        logger.write_accross(arr, next_ind, target, ind);
    }
    for ind in (start_b..len).step_by(jump_b) {
        let next_ind: usize = if tmp_start_a < tmp_start_b {
            tmp_start_a += jump_a;
            tmp_start_a - jump_a
        } else {
            tmp_start_b += jump_b;
            tmp_start_b - jump_b
        };
        logger.write_accross(arr, next_ind, target, ind);
    }
}

fn jump_merge_2<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    target: &mut [T],
    arr: &[T],
    ind_a: usize,
    ind_b: usize,
    jump: usize,
    logger: &mut U,
) {
    //println!("{}, {}, {}, {}", ind_a, ind_b, jump, arr.len());
    let (mut ind_a, mut ind_b) = (ind_a, ind_b);
    let mut tmp = ind_a;
    while ind_a < arr.len() && ind_b < arr.len() {
        if logger.cmp_lt(arr, ind_a, ind_b) {
            logger.write_accross(arr, ind_a, target, tmp);
            ind_a += jump;
        } else {
            logger.write_accross(arr, ind_b, target, tmp);
            ind_b += jump;
        }
        tmp += jump / 2;
    }
    while ind_a < arr.len() {
        logger.write_accross(arr, ind_a, target, tmp);
        ind_a += jump;
        tmp += jump / 2;
    }
    while ind_b < arr.len() {
        logger.write_accross(arr, ind_b, target, tmp);
        ind_b += jump;
        tmp += jump / 2;
    }
}

use crate::traits::log_traits::SortLogger;
pub fn merge_inplace<T: Ord + Copy, U: SortLogger<T>>(
    arr: &[T],
    target: &mut [T],
    start: usize,
    jump: usize,
    logger: &mut U,
) {
    let mut ind_a = start;
    let mut ind_b = start + jump;
    let mut ind_c = start;
    while ind_a < arr.len() && ind_b < arr.len() {
        if logger.cmp_le(arr, ind_a, ind_b) {
            logger.write_accross(arr, ind_a, target, ind_c);
            ind_c += jump;
            ind_a += jump * 2;
        } else {
            logger.write_accross(arr, ind_b, target, ind_c);
            ind_c += jump;
            ind_b += jump * 2;
        }
    }
    while ind_a < arr.len() {
        logger.write_accross(arr, ind_a, target, ind_c);
        ind_a += jump * 2;
        ind_c += jump;
    }
    while ind_b < arr.len() {
        logger.write_accross(arr, ind_b, target, ind_c);
        ind_b += jump * 2;
        ind_c += jump;
    }
}

pub fn merge_jump<T: Ord + Copy, U: SortLogger<T>>(
    arr: &[T],
    target: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    //println!("start: {}, {jump}", arr.len());
    let mut a = 0;
    let mut b = jump;
    let mut c = 0;
    while a < arr.len() && b < arr.len() {
        if logger.cmp_lt(arr, a, b) {
            logger.write_accross(arr, a, target, c);
            a += jump * 2;
            c += jump;
        } else {
            logger.write_accross(arr, b, target, c);
            b += jump * 2;
            c += jump;
        }
    }
    while a < arr.len() {
        //println!("a {a}, {c}, {jump}");
        logger.write_accross(arr, a, target, c);
        a += jump * 2;
        c += jump;
    }
    while b < arr.len() {
        //println!("b {b}, {c}, {jump}");
        logger.write_accross(arr, b, target, c);
        b += jump * 2;
        c += jump;
    }
}
