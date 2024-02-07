pub mod heap_quick_sort;
pub mod heap_quick_sort_optimized;
pub mod heap_quick_sort_optimized_tmp;

use crate::traits::{SortAlgo, SortLogger};
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        type Sort<A, B> = heap_quick_sort::SortImp<A, B>;
        Sort::<T, U>::sort(arr, logger);
        vec![format!("name: {}", Sort::<T, U>::name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "heap_quick_sort_optimized" => {
                type Sort<A, B> = heap_quick_sort_optimized::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "heap_quick_sort_optimized_tmp" => {
                type Sort<A, B> = heap_quick_sort_optimized_tmp::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "heap_quick_sort" | _ => {
                type Sort<A, B> = heap_quick_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "heap_quick_sort",
            "heap_quick_sort_optimized",
            "heap_quick_sort_optimized_tmp",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
