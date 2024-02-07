pub mod comb_classic;
pub mod comb_random_gaps;

use crate::traits::{SortAlgo, SortLogger};
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        type Sort<A, B> = comb_classic::SortImp<A, B>;
        Sort::<T, U>::sort(arr, logger);
        vec![format!("name: {}", Sort::<T, U>::name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "comb_random_gaps" => {
                type Sort<A, B> = comb_random_gaps::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}\n", Sort::<T, U>::name())]
            }
            "comb_classic" | _ => {
                type Sort<A, B> = comb_classic::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}\n", Sort::<T, U>::name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        ["comb_classic", "comb_random_gaps"]
            .iter()
            .map(|i| i.to_string())
            .collect()
    } else {
        vec![]
    }
}
