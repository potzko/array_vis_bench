use crate::traits::{SortAlgo, SortLogger};

pub mod insertion_sort;

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    _: &[String],
) -> Vec<String> {
    type Sort<A, B> = insertion_sort::SortImp<A, B>;
    Sort::sort(arr, logger);
    vec![format!("name: {}", Sort::<T, U>::name())]
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        vec!["insertion_sort".to_string()]
    } else {
        vec![]
    }
}
