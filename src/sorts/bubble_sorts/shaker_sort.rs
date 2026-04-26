use crate::traits::log_traits::SortLogger;

pub struct ShakerSort;

impl ShakerSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
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
}

sort_registry_macro::sort_family! {
    type Sort = ShakerSort;
    name        = "shaker sort";
    big_o       = "O(N^2)";
    stable      = true;
    direct_sort = true;
    path        = ["bubble sorts", "shaker sort"];
}
