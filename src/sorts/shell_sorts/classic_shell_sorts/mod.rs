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
        type Sort<A, B> = shell_classic::SortImp<A, B>;
        Sort::sort(arr, logger);
        vec![format!("name: {}", Sort::<T, U>::name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "shell_classic_ordered_insertion" => {
                type Sort<A, B> = shell_classic_ordered_insertion::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_hibbard" => {
                type Sort<A, B> = shell_hibbard::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_knuth" => {
                type Sort<A, B> = shell_knuth::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_optimized_256_elements" => {
                type Sort<A, B> = shell_optimized_256_elements::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_sedgewick" => {
                type Sort<A, B> = shell_sedgewick::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_sedgewick_branching" => {
                type Sort<A, B> = shell_sedgewick_branching::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_sedgewick_ordered_insertion" => {
                type Sort<A, B> = shell_sedgewick_ordered_insertion::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shell_classic" | _ => {
                type Sort<A, B> = shell_classic::SortImp<A, B>;
                Sort::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
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
