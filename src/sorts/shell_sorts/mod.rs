pub mod combinations;
pub mod sequences;
pub mod shell_sort;
pub mod shell_sort_ordered;

// Old submodules — disconnected during refactor, see REFACTOR_PLAN.md.
// Uncomment to restore (note: old implementations use the deprecated write/cmp API).
// pub mod classic_shell_sorts;
// pub mod shell_shell_sorts;

use crate::traits::log_traits::SortLogger;
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
/// shell-sort variant (i.e. it exists in `GAP_SEQUENCES`), otherwise `None`.
///
/// Using this instead of a hardcoded prefix check in `main.rs` ensures that
/// any new variant added via `register_sequence!` is automatically routable
/// without touching the dispatch logic.
pub fn sort_choice(name: &str) -> Option<Vec<String>> {
    for entry in GAP_SEQUENCES {
        if entry.name == name {
            return Some(vec!["shell_sorts".to_string(), name.to_string()]);
        }
    }
    None
}
