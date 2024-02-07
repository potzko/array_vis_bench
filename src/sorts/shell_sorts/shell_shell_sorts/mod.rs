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
        type Sort<A, B> = shell_shell_sort_classic::SortImp<A, B>;
        Sort::sort(arr, logger);
        vec![format!("name: {}", Sort::<T, U>::name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "shell_shell_sort_3parity" => {
                type Sort<A, B> = shell_shell_sort_3parity::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_shell_sort_fibonacci" => {
                type Sort<A, B> = shell_shell_sort_fibonacci::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_shell_sort_log_parity" => {
                type Sort<A, B> = shell_shell_sort_log_parity::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_shell_sort_optimised" => {
                type Sort<A, B> = shell_shell_sort_optimised::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_shell_sort_root_parity" => {
                type Sort<A, B> = shell_shell_sort_root_parity::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_shell_sort_classic" | _ => {
                type Sort<A, B> = shell_shell_sort_classic::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
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
