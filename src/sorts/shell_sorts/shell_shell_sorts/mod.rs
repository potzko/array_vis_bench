pub mod shell_shell_sort_3parity;
pub mod shell_shell_sort_classic;
pub mod shell_shell_sort_fibonacci;
pub mod shell_shell_sort_log_parity;
pub mod shell_shell_sort_optimised;
pub mod shell_shell_sort_root_parity;

use crate::traits::{SortAlgo, SortLogger};
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        let sort = shell_shell_sort_classic::ShellSort {};
        sort.sort(arr, logger);
        vec![format!("name: {}", sort.name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "shell_shell_sort_3parity" => {
                let sort = shell_shell_sort_3parity::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_shell_sort_fibonacci" => {
                let sort = shell_shell_sort_fibonacci::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_shell_sort_log_parity" => {
                let sort = shell_shell_sort_log_parity::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_shell_sort_optimised" => {
                let sort = shell_shell_sort_optimised::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_shell_sort_root_parity" => {
                let sort = shell_shell_sort_root_parity::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_shell_sort_classic" | _ => {
                let sort = shell_shell_sort_classic::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "shell_shell_sort_3parity",
            "shell_shell_sort_classic",
            "shell_shell_sort_fibonacci",
            "shell_shell_sort_log_parity",
            "shell_shell_sort_optimised",
            "shell_shell_sort_root_parity",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
