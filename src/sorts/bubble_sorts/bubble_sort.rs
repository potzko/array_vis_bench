use crate::traits::log_traits::SortLogger;

pub struct BubbleSort;

impl BubbleSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        for i in 0..arr.len() {
            for ii in 1..arr.len() - i {
                logger.cond_swap_lt(arr, ii, ii - 1);
            }
        }
    }
}

sort_registry_macro::sort_family! {
    type Sort = BubbleSort;
    name        = "bubble sort";
    big_o       = "O(N^2)";
    stable      = true;
    direct_sort = true;
    path        = ["bubble sorts", "bubble sort"];
}
