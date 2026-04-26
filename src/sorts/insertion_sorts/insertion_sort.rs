use crate::traits::log_traits::SortLogger;

pub struct InsertionSort;

impl InsertionSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        crate::utils::small_sort::insertion_sort(arr, logger);
    }
}

sort_registry_macro::sort_family! {
    type Sort = InsertionSort;
    name        = "insertion sort";
    big_o       = "O(N^2)";
    stable      = true;
    direct_sort = true;
    path        = ["insertion sorts", "insertion sort"];
}
