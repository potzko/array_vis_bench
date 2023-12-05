use crate::traits::{SortAlgo, SortLogger};

pub mod bubble_sort;
pub mod bubble_sort_recursive;
pub mod odd_even_bubble_sort;
pub mod shaker_sort;

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        let sort = bubble_sort::BubbleSort {};
        sort.sort(arr, logger);
        vec![format!("name: {}", sort.name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "bubble_sort_recursive" => {
                let sort = bubble_sort_recursive::BubbleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "odd_even_bubble_sort" => {
                let sort = odd_even_bubble_sort::BubbleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shaker_sort" => {
                let sort = shaker_sort::BubbleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "bubble_sort" | _ => {
                let sort = bubble_sort::BubbleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "bubble_sort",
            "bubble_sort_recursive",
            "shaker_sort",
            "bubble_sort",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
