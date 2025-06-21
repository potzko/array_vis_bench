use crate::create_sort;

create_sort!(sort, "shaker sort", "O(N^2)", true);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut left = 0;
    let mut right = arr.len() - 1;
    while left < right {
        for i in left + 1..=right {
            logger.cond_swap_le(arr, i, i - 1);
        }
        right -= 1;
        for i in (left + 1..=right).rev() {
            logger.cond_swap_le(arr, i, i - 1);
        }
        left += 1;
    }
}
