pub mod classic_merge_sort;
pub mod merge_sort_bottom_up;
pub mod merge_sort_bottom_up_optimized;
pub mod merge_sort_optimized;
pub mod merge_sort_outside_lists;
mod utils;

use crate::traits::{SortAlgo, SortLogger};
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        type Sort<A, B> = classic_merge_sort::SortImp<A, B>;
        Sort::<T, U>::sort(arr, logger);
        vec![format!("name: {}", Sort::<T, U>::name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "merge_sort_bottom_up" => {
                type Sort<A, B> = merge_sort_bottom_up::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "merge_sort_bottom_up_optimized" => {
                type Sort<A, B> = merge_sort_bottom_up_optimized::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "merge_sort_optimized" => {
                type Sort<A, B> = merge_sort_optimized::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "merge_sort_outside_lists" => {
                type Sort<A, B> = merge_sort_outside_lists::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "classic_merge_sort" | _ => {
                type Sort<A, B> = classic_merge_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "classic_merge_sort",
            "merge_sort_bottom_up",
            "merge_sort_bottom_up_optimized",
            "merge_sort_outside_lists",
            "classic_merge_sort",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
