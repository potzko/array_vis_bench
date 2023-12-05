pub mod classic_shell_sorts;
pub mod shell_shell_sorts;

use crate::traits::SortLogger;
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        classic_shell_sorts::fn_sort(arr, logger, choice)
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "shell_shell_sorts" => shell_shell_sorts::fn_sort(arr, logger, &choice[1..]),
            "classic_shell_sorts" | _ => classic_shell_sorts::fn_sort(arr, logger, &choice[1..]),
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        ["shell_shell_sorts", "classic_shell_sorts"]
            .iter()
            .map(|i| i.to_string())
            .collect()
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "shell_shell_sorts" => shell_shell_sorts::options(&choice[1..]),
            "classic_shell_sorts" | _ => classic_shell_sorts::options(&choice[1..]),
        }
    }
}
