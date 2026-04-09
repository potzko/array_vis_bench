use crate::create_sort;
use crate::traits::log_traits::SortLogger;

create_sort!(sort, "shaker sort", "O(N^2)", true);

fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    if arr.len() < 2 {
        return;
    }
    let mut left = 0;
    let mut right = arr.len() - 1;
    while left < right {
        for i in left + 1..=right {
            logger.cond_swap_lt(arr, i, i - 1);
        }
        right -= 1;
        for i in (left + 1..=right).rev() {
            logger.cond_swap_lt(arr, i, i - 1);
        }
        left += 1;
    }
}

pub fn sort_dyn(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
    sort(arr, logger);
}
