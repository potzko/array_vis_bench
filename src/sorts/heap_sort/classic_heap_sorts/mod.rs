use crate::traits::{SortAlgo, SortLogger};

pub mod base_16_heap;
pub mod base_256_heap;
pub mod base_3_heap;
pub mod heap_sort_classic;

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        let sort = heap_sort_classic::HeapSort {};
        sort.sort(arr, logger);
        vec![format!("name: {}", sort.name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "base_3_heap" => {
                let sort = base_3_heap::HeapSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "base_256_heap" => {
                let sort = base_256_heap::HeapSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "base_16_heap" => {
                let sort = base_3_heap::HeapSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "base_2_heap" | "heap_sort_classic" | _ => {
                let sort = base_256_heap::HeapSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "base_16_heap",
            "base_3_heap",
            "base_256_heap",
            "base_2_heap",
            "heap_sort_classic",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
