use crate::create_sort;
use crate::traits::sort_traits::SortAlgo;

create_sort!(
    sort,
    "circle sort bottom up increasing size",
    "O(N^log^2(N))",
    false
);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    for _ in 0..(((arr.len()) as f64).log2() as usize) >> 1 {
        if !circle_sort_iteration(arr, logger) {
            break;
        }
    }
    type SmallSort<A, B> = crate::sorts::insertion_sorts::insertion_sort::SortImp<A, B>;
    SmallSort::sort(arr, logger);
}

fn circle_sort_iteration<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> bool {
    let mut swapped = false;
    let mut iter = get_last_bit(arr.len());
    while iter > 1 {
        for i in (0..arr.len()).step_by(iter) {
            let mut ind_left = i;
            let mut ind_right = i + iter - 1;
            if ind_right >= arr.len() {
                let tmp = ind_right - arr.len() + 1;
                ind_left += tmp;
                ind_right -= tmp;
            }
            while ind_left < ind_right {
                if logger.cond_swap_lt(arr, ind_right, ind_left) {
                    swapped = true
                }
                ind_left += 1;
                ind_right -= 1;
            }
        }
        iter /= 2;
    }
    swapped
}

fn get_last_bit(n: usize) -> usize {
    1 << ((usize::BITS - n.leading_zeros() - 1) as usize)
}
