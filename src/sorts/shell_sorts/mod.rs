pub mod branching;
pub mod combinations;
pub mod sequences;
pub mod shell_shell_sort;
pub mod shell_sort;
pub mod shell_sort_ordered;

use crate::traits::log_traits::SortLogger;
use branching::BRANCHING_STRATEGIES;
use sequences::{Classic, GAP_SEQUENCES};
use shell_sort::ShellSort;

pub fn fn_sort(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
    choice: &[String],
) -> Vec<String> {
    let name = choice.first().map(String::as_str).unwrap_or("");

    for entry in GAP_SEQUENCES {
        if entry.name == name {
            (entry.sort_vis)(arr, logger);
            return vec![format!("name: {}", name)];
        }
    }
    for entry in BRANCHING_STRATEGIES {
        if entry.name == name {
            (entry.sort_vis)(arr, logger);
            return vec![format!("name: {}", name)];
        }
    }

    // Default: classic sequence
    ShellSort::<Classic>::sort(arr, logger);
    vec![format!("name: {}", name)]
}

pub fn options(_: &[String]) -> Vec<String> {
    // Sort discovery goes through get_registered_sorts() in the new architecture.
    // This stub is kept for legacy compatibility with sorts::options().
    vec![]
}

/// Returns the choice vec for `fn_sort` dispatch if `name` is a registered
/// shell-sort or shell-shell-sort variant, otherwise `None`.
///
/// Checks both `GAP_SEQUENCES` and `BRANCHING_STRATEGIES` so any new variant
/// added via `register_sequence!` or `register_branching!` is automatically
/// routable without touching `main.rs`.
pub fn sort_choice(name: &str) -> Option<Vec<String>> {
    for entry in GAP_SEQUENCES {
        if entry.name == name {
            return Some(vec!["shell_sorts".to_string(), name.to_string()]);
        }
    }
    for entry in BRANCHING_STRATEGIES {
        if entry.name == name {
            return Some(vec!["shell_sorts".to_string(), name.to_string()]);
        }
    }
    None
}
