pub mod bad_heap_sort;
pub mod bad_heap_sort_alt;
pub mod cyclent_sort;
pub mod cyclent_sort_stack;
pub mod cyclent_sort_stack_optimized;
pub mod random_shell_sort;
pub mod slow_sort;
pub mod stooge_sort;
pub mod cyclent_sort_opt;
pub mod slow_sort_potzko;
pub mod quick_surrender;
pub mod quick_surrender_optimised;

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/fun_sorts_combinations.rs"));
}

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
    if name == "bad heap sort"
        || name == "bad heap sort alt"
        || name == "random shell sort"
        || name.starts_with("cyclent sort")
        || name.starts_with("cyclent sort stack optimized")
        || name.starts_with("cyclent sort opt")
        || name.starts_with("slow sort")
        || name.starts_with("slow sort potzko")
        || name.starts_with("stooge sort")
        || name.starts_with("quick surrender")
    {
        return Some(vec!["fun_sorts".to_string(), name.to_string()]);
    }
    None
}
