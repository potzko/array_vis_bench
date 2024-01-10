pub mod classic_heap_sorts;
pub mod heap_quick_sort;
pub mod quick_heap_sorts;
pub mod weak_heap_sort;

use crate::traits::{SortAlgo, SortLogger};

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        classic_heap_sorts::fn_sort(arr, logger, choice)
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "weak_heap_sort" => {
                let sort = weak_heap_sort::HeapSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "heap_quick_sort" => {
                let sort = heap_quick_sort::HeapSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "quick_heap_sorts" => quick_heap_sorts::fn_sort(arr, logger, &choice[1..]),

            "classic_heap_sorts" | _ => classic_heap_sorts::fn_sort(arr, logger, &choice[1..]),
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "weak_heap_sort",
            "quick_heap_sorts",
            "classic_heap_sorts",
            "heap_quick_sort",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "quick_heap_sorts" => quick_heap_sorts::options(&choice[1..]),
            "classic_heap_sorts" => classic_heap_sorts::options(&choice[1..]),
            _ => vec![],
        }
    }
}
