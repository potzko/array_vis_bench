pub mod shell_classic;
pub mod shell_classic_ordered_insertion;
pub mod shell_hibbard;
pub mod shell_knuth;
pub mod shell_optimized_256_elements;
pub mod shell_sedgewick;
pub mod shell_sedgewick_branching;
pub mod shell_sedgewick_ordered_insertion;

use crate::traits::{SortAlgo, SortLogger};
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        let sort = shell_classic::ShellSort {};
        sort.sort(arr, logger);
        vec![format!("name: {}", sort.name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "shell_classic_ordered_insertion" => {
                let sort = shell_classic_ordered_insertion::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_hibbard" => {
                let sort = shell_hibbard::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_knuth" => {
                let sort = shell_knuth::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_optimized_256_elements" => {
                let sort = shell_optimized_256_elements::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_sedgewick" => {
                let sort = shell_sedgewick::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_sedgewick_branching" => {
                let sort = shell_sedgewick_branching::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_sedgewick_ordered_insertion" => {
                let sort = shell_sedgewick_ordered_insertion::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shell_classic" | _ => {
                let sort = shell_classic::ShellSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "shell_classic",
            "shell_classic_ordered_insertion",
            "shell_hibbard",
            "shell_knuth",
            "shell_optimized_256_elements",
            "shell_sedgewick",
            "shell_sedgewick_branching",
            "shell_sedgewick_ordered_insertion",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
