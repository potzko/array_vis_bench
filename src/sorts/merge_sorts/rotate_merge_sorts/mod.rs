pub mod rotate_merge_sort;
pub mod rotate_merge_sort_bottom_up;
pub mod rotate_merge_sort_bottom_up_optimized;
pub mod rotate_merge_sort_optimized;
mod utils;

use crate::traits::{SortAlgo, SortLogger};
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        let sort = rotate_merge_sort::MergeSort {};
        sort.sort(arr, logger);
        vec![format!("name: {}", sort.name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "rotate_merge_sort_optimized" => {
                let sort = rotate_merge_sort_optimized::MergeSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "rotate_merge_sort_bottom_up_optimized" => {
                let sort = rotate_merge_sort_bottom_up_optimized::MergeSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "rotate_merge_sort_bottom_up" => {
                let sort = rotate_merge_sort_bottom_up::MergeSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "rotate_merge_sort" | _ => {
                let sort = rotate_merge_sort::MergeSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "rotate_merge_sort",
            "rotate_merge_sort_bottom_up",
            "rotate_merge_sort_bottom_up_optimized",
            "rotate_merge_sort_optimized",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
