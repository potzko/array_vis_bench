pub mod heap_quick_sort;
pub mod heap_quick_sort_optimized;

use crate::traits::{SortAlgo, SortLogger};
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        let sort = heap_quick_sort::HeapSort {};
        sort.sort(arr, logger);
        vec![format!("name: {}", sort.name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "heap_quick_sort_optimized" => {
                let sort = heap_quick_sort_optimized::HeapSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "heap_quick_sort" | _ => {
                let sort = heap_quick_sort::HeapSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        ["heap_quick_sort", "heap_quick_sort_optimized"]
            .iter()
            .map(|i| i.to_string())
            .collect()
    } else {
        vec![]
    }
}
