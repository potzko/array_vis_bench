pub mod comb_classic;
pub mod comb_random_gaps;

use crate::traits::{SortAlgo, SortLogger};
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        let sort = comb_classic::CombSort {};
        sort.sort(arr, logger);
        vec![format!("name: {}", sort.name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "comb_random_gaps" => {
                let sort = comb_random_gaps::CombSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}\n", sort.name())]
            }
            "comb_classic" | _ => {
                let sort = comb_classic::CombSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}\n", sort.name())]
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
