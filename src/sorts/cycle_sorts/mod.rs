use crate::traits::{SortAlgo, SortLogger};

pub mod cycle_sort;

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    _: &[String],
) -> Vec<String> {
    cycle_sort::SortImp::<T, U>::sort(arr, logger);
    vec![format!("name: {}", cycle_sort::SortImp::<T, U>::name())]
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        vec!["cycle_sort".to_string()]
    } else {
        vec![]
    }
}
