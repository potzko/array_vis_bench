pub mod midian_pivot_quick_sort;
pub mod quick_sort_left_left_pointers;
pub mod quick_sort_left_left_pointers_optimised;
pub mod quick_sort_left_right_pivot_optimised;
pub mod quick_sort_left_right_pointers_moving_pivot;
pub mod quick_sort_left_right_pointers_static_pivot;

use crate::traits::{SortAlgo, SortLogger};
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        let sort = quick_sort_left_right_pointers_static_pivot::QuickSort {};
        sort.sort(arr, logger);
        vec![format!("name: {}", sort.name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "midian_pivot_quick_sort" => {
                let sort = midian_pivot_quick_sort::QuickSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "quick_sort_left_left_pointers" => {
                let sort = quick_sort_left_left_pointers::QuickSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "quick_sort_left_left_pointers_optimised" => {
                let sort = quick_sort_left_left_pointers_optimised::QuickSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "quick_sort_left_right_pivot_optimised" => {
                let sort = quick_sort_left_right_pivot_optimised::QuickSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "quick_sort_left_right_pointers_moving_pivot" => {
                let sort = quick_sort_left_right_pointers_moving_pivot::QuickSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "quick_sort_left_right_pointers_static_pivot"
            | "quick_sort_left_right_pointers"
            | _ => {
                let sort = quick_sort_left_right_pointers_static_pivot::QuickSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "midian_pivot_quick_sort",
            "quick_sort_left_left_pointers",
            "quick_sort_left_left_pointers_optimised",
            "quick_sort_left_right_pivot_optimised",
            "quick_sort_left_right_pointers_moving_pivot",
            "quick_sort_left_right_pointers_static_pivot",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
