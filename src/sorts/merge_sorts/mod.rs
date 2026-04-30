mod utils;
pub mod bottom_up;
pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/merge_sorts_combinations.rs"));
}
pub mod naive;
pub mod natural;
pub mod rotation;
pub mod rotation_merge;
pub mod top_down;

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
    if crate::traits::SORT_VIS_REGISTRY.lock().unwrap().contains_key(name) {
        return Some(vec!["merge_sorts".to_string(), name.to_string()]);
    }
    None
}
