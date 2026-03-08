pub mod branching;
pub mod combinations;
pub mod rod_sort;

use crate::traits::log_traits::SortLogger;
use branching::ROD_STRATEGIES;

pub fn fn_sort(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
    choice: &[String],
) -> Vec<String> {
    let name = choice.first().map(String::as_str).unwrap_or("");

    for entry in ROD_STRATEGIES {
        if entry.name == name {
            (entry.sort_vis)(arr, logger);
            return vec![format!("name: {}", name)];
        }
    }

    // Default: classic strategy
    use branching::Classic;
    use rod_sort::RodSort;
    RodSort::<Classic>::sort(arr, logger);
    vec![format!("name: {}", name)]
}

/// Returns the choice vec for `fn_sort` dispatch if `name` is a registered
/// rod-sort variant, otherwise `None`.
pub fn sort_choice(name: &str) -> Option<Vec<String>> {
    for entry in ROD_STRATEGIES {
        if entry.name == name {
            return Some(vec!["rod_sorts".to_string(), name.to_string()]);
        }
    }
    None
}
