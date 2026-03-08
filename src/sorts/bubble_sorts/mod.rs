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
    match name {
        "bubble sort recursive" => bubble_sort_recursive::sort_dyn(arr, logger),
        "odd-even bubble sort"  => odd_even_bubble_sort::sort_dyn(arr, logger),
        "shaker sort"           => shaker_sort::sort_dyn(arr, logger),
        _                       => bubble_sort::sort_dyn(arr, logger),
    }
    vec![format!("name: {}", name)]
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
