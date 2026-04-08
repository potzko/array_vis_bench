use crate::traits::log_traits::SortLogger;

pub mod insertion_sort;

pub fn fn_sort(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
    _: &[String],
) -> Vec<String> {
    insertion_sort::sort_dyn(arr, logger);
    vec!["name: insertion sort".to_string()]
}

pub fn sort_choice(name: &str) -> Option<Vec<String>> {
    match name {
        "insertion sort" => Some(vec!["insertion_sorts".to_string(), name.to_string()]),
        _ => None,
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        vec!["insertion_sort".to_string()]
    } else {
        vec![]
    }
}
