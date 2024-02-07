pub mod iterative_quick_sort;
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
        type Sort<A, B> = quick_sort_left_right_pointers_static_pivot::SortImp<A, B>;
        Sort::sort(arr, logger);
        vec![format!("name: {}", Sort::<T, U>::name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "midian_pivot_quick_sort" => {
                type Sort<A, B> = midian_pivot_quick_sort::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "quick_sort_left_left_pointers" => {
                type Sort<A, B> = quick_sort_left_left_pointers::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "quick_sort_left_left_pointers_optimised" => {
                type Sort<A, B> = quick_sort_left_left_pointers_optimised::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "quick_sort_left_right_pivot_optimised" => {
                type Sort<A, B> = quick_sort_left_right_pivot_optimised::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "quick_sort_left_right_pointers_moving_pivot" => {
                type Sort<A, B> = quick_sort_left_right_pointers_moving_pivot::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "iterative_quick_sort" => {
                type Sort<A, B> = iterative_quick_sort::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "quick_sort_left_right_pointers_static_pivot"
            | "quick_sort_left_right_pointers"
            | _ => {
                type Sort<A, B> = quick_sort_left_right_pointers_static_pivot::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
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
            "iterative_quick_sort",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
