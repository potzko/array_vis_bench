use crate::traits::log_traits::SortLogger;

fn swap_section<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    a: usize,
    b: usize,
    len: usize,
    logger: &mut U,
) {
    for i in 0..len {
        logger.swap(arr, a + i, b + i);
    }
}

#[allow(dead_code)]
pub fn rotate_2<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    mut a: usize,
    mut m: usize,
    mut b: usize,
    logger: &mut U,
) {
    let mut l = m - a;
    let mut r = b - m;

    while l > 0 && r > 0 {
        if r < l {
            swap_section(arr, m - r, m, r, logger);
            b -= r;
            m -= r;
            l -= r;
        } else {
            swap_section(arr, a, m, l, logger);
            a += l;
            m += l;
            r -= l;
        }
    }
}

fn reverse<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for i in 0..arr.len() / 2 {
        logger.swap(arr, i, arr.len() - i - 1);
    }
}

pub fn rotate<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], split_ind: usize, logger: &mut U) {
    reverse(&mut arr[..split_ind], logger);
    reverse(&mut arr[split_ind..], logger);
    reverse(arr, logger);
}
