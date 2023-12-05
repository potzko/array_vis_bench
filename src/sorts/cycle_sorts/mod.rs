use crate::traits::{SortAlgo, SortLogger};

pub mod cycle_sort;

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    _: &[String],
) -> Vec<String> {
    let sort = cycle_sort::CycleSort {};
    sort.sort(arr, logger);
    vec![format!("name: {}", sort.name())]
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        vec!["cycle_sort".to_string()]
    } else {
        vec![]
    }
}
