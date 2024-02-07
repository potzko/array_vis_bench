use crate::traits::{SortAlgo, SortLogger};

pub mod bad_heap_sort;
pub mod bad_heap_sort_alt;
pub mod cyclent_sort;
pub mod cyclent_sort_stack;
pub mod cyclent_sort_stack_optimized;
pub mod quick_surrender;
pub mod quick_surrender_optimised;
pub mod random_shell_sort;
pub mod slow_sort;
pub mod stooge_sort;

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        type Sort<A, B> = cyclent_sort::SortImp<A, B>;
        Sort::<T, U>::sort(arr, logger);
        vec![format!("name: {}", Sort::<T, U>::name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "bad_heap_sort" => {
                type Sort<A, B> = bad_heap_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "bad_heap_sort_alt" => {
                type Sort<A, B> = bad_heap_sort_alt::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "cyclent_sort_stack" => {
                type Sort<A, B> = cyclent_sort_stack::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "random_shell_sort" => {
                type Sort<A, B> = random_shell_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "slow_sort" => {
                type Sort<A, B> = slow_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "stooge_sort" => {
                type Sort<A, B> = stooge_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "cyclent_sort" | _ => {
                type Sort<A, B> = cyclent_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "bad_heap_sort",
            "bad_heap_sort_alt",
            "cyclent_sort",
            "cyclent_sort_stack",
            "cyclent_sort_stack_optimized",
            "quick_surrender",
            "quick_surrender_optimised",
            "random_shell_sort",
            "slow_sort",
            "stooge_sort",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
