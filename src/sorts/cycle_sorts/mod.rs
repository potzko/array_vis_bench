pub mod cycle_sort;

use crate::traits::log_traits::SortLogger;

pub fn fn_sort(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
    choice: &[String],
) -> Vec<String> {
    let name = choice.first().map(String::as_str).unwrap_or("");
    cycle_sort::sort_dyn(arr, logger);
    vec![format!("name: {}", name)]
}

pub fn sort_choice(name: &str) -> Option<Vec<String>> {
    if name == "cycle sort" {
        Some(vec!["cycle_sorts".to_string(), name.to_string()])
    } else {
        None
    }
}
