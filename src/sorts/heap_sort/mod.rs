pub mod classic_heap_sorts;
pub mod smooth_sort;
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
            "smooth_sort" => {
                let sort = smooth_sort::HeapSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "weak_heap_sort" => {
                let sort = weak_heap_sort::HeapSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "classic_heap_sorts" | _ => classic_heap_sorts::fn_sort(arr, logger, &choice[1..]),
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        ["smooth_sort", "weak_heap_sort", "classic_heap_sorts"]
            .iter()
            .map(|i| i.to_string())
            .collect()
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "classic_heap_sorts" | _ => classic_heap_sorts::options(choice),
        }
    }
}
