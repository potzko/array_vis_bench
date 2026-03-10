mod utils;
pub mod small_sort;
pub mod bottom_up;
pub mod combinations;
pub mod natural;
pub mod rotation;
pub mod rotation_merge;
pub mod top_down;
pub mod top_down_mirror;

use crate::traits::log_traits::SortLogger;
use combinations::MERGE_SORTS;

pub fn fn_sort(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
    choice: &[String],
) -> Vec<String> {
    let name = choice.first().map(String::as_str).unwrap_or("");
    for entry in MERGE_SORTS {
        if entry.name == name {
            (entry.sort_vis)(arr, logger);
            return vec![format!("name: {}", name)];
        }
    }
    vec![format!("name: {} (not found)", name)]
}

pub fn sort_choice(name: &str) -> Option<Vec<String>> {
    for entry in MERGE_SORTS {
        if entry.name == name {
            return Some(vec!["merge_sorts".to_string(), name.to_string()]);
        }
    }
    None
}
