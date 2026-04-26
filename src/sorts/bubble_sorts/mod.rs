pub mod bubble_sort;
pub mod bubble_sort_recursive;
pub mod odd_even_bubble_sort;
pub mod shaker_sort;

use crate::traits::log_traits::SortLogger;

pub fn fn_sort(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
    choice: &[String],
) -> Vec<String> {
    let name = choice.first().map(String::as_str).unwrap_or("");
    if let Some(vis_fn) = crate::traits::SORT_VIS_REGISTRY.lock().unwrap().get(name).copied() {
        vis_fn(arr, logger);
        return vec![format!("name: {}", name)];
    }
    vec![format!("name: {} (not found)", name)]
}

pub fn sort_choice(name: &str) -> Option<Vec<String>> {
    match name {
        "bubble sort"
        | "bubble sort recursive"
        | "odd-even bubble sort"
        | "shaker sort" => Some(vec!["bubble_sorts".to_string(), name.to_string()]),
        _ => None,
    }
}
