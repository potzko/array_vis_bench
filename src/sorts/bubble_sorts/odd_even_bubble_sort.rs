use crate::traits::log_traits::SortLogger;

pub struct OddEvenBubbleSort;

impl OddEvenBubbleSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let mut sorted = false;
        while !sorted {
            sorted = true;
            for i in (1..arr.len()).step_by(2) {
                if logger.cond_swap_lt(arr, i, i - 1) {
                    sorted = false;
                }
            }
            for i in (2..arr.len()).step_by(2) {
                if logger.cond_swap_lt(arr, i, i - 1) {
                    sorted = false;
                }
            }
        }
    }
}

sort_registry_macro::sort_family! {
    type Sort = OddEvenBubbleSort;
    name        = "odd-even bubble sort";
    big_o       = "O(N^2)";
    stable      = true;
    direct_sort = true;
    path        = ["bubble sorts", "odd-even bubble sort"];
}
