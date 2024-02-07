use crate::{traits::SortAlgo, traits::SortLogger};

pub mod circle_sort_bottom_up;
pub mod circle_sort_bottom_up_increasing_size;
pub mod circle_sort_bottom_up_optimised;
pub mod circle_sort_recursive;
pub mod circle_sort_recursive_increasing_size;
pub mod circle_sort_recursive_optimised;
pub mod shaker_circle_sort;
pub mod shaker_circle_sort_bottom_up;
pub mod shaker_circle_sort_bottom_up_b;
pub mod stooge_circle_sort;
pub mod stooge_circle_sort_reversed;

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        type Sort<A, B> = circle_sort_recursive::SortImp<A, B>;
        Sort::<T, U>::sort(arr, logger);
        vec![format!("name: {}", Sort::<T, U>::name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "circle_sort_bottom_up" => {
                type Sort<A, B> = circle_sort_bottom_up::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "circle_sort_bottom_up_increasing_size" => {
                type Sort<A, B> = circle_sort_bottom_up_increasing_size::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "circle_sort_bottom_up_optimised" => {
                type Sort<A, B> = circle_sort_bottom_up_optimised::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "circle_sort_recursive_increasing_size" => {
                type Sort<A, B> = circle_sort_recursive_increasing_size::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "circle_sort_recursive_optimised" => {
                type Sort<A, B> = circle_sort_recursive_optimised::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shaker_circle_sort" => {
                type Sort<A, B> = shaker_circle_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shaker_circle_sort_bottom_up" => {
                type Sort<A, B> = shaker_circle_sort_bottom_up::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shaker_circle_sort_bottom_up_b" => {
                type Sort<A, B> = shaker_circle_sort_bottom_up_b::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "stooge_circle_sort" => {
                type Sort<A, B> = stooge_circle_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "stooge_circle_sort_reversed" => {
                type Sort<A, B> = stooge_circle_sort_reversed::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "circle_sort_recursive" | _ => {
                type Sort<A, B> = circle_sort_recursive::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "circle_sort_bottom_up",
            "circle_sort_bottom_up_increasing_size",
            "circle_sort_bottom_up_optimised",
            "circle_sort_recursive",
            "circle_sort_recursive_increasing_size",
            "circle_sort_recursive_optimised",
            "shaker_circle_sort",
            "shaker_circle_sort_bottom_up",
            "shaker_circle_sort_bottom_up_b",
            "stooge_circle_sort",
            "stooge_circle_sort_reversed",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
