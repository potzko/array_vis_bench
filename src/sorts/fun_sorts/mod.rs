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
        let sort = cyclent_sort::FunSort {};
        sort.sort(arr, logger);
        vec![format!("name: {}", sort.name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "bad_heap_sort" => {
                let sort = bad_heap_sort::FunSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "bad_heap_sort_alt" => {
                let sort = bad_heap_sort_alt::FunSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "cyclent_sort_stack" => {
                let sort = cyclent_sort_stack::FunSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "random_shell_sort" => {
                let sort = random_shell_sort::FunSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "slow_sort" => {
                let sort = slow_sort::FunSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "stooge_sort" => {
                let sort = stooge_sort::FunSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "cyclent_sort" | _ => {
                let sort = cyclent_sort::FunSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
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
