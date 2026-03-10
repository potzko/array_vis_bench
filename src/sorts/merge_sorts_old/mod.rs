pub mod classic_merge_sorts;
pub mod rotate_merge_sorts;

use crate::traits::SortLogger;
pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        classic_merge_sorts::fn_sort(arr, logger, choice)
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "rotate_merge_sorts" => rotate_merge_sorts::fn_sort(arr, logger, &choice[1..]),
            "classic_merge_sorts" | _ => classic_merge_sorts::fn_sort(arr, logger, &choice[1..]),
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        ["rotate_merge_sorts", "classic_merge_sorts"]
            .iter()
            .map(|i| i.to_string())
            .collect()
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "rotate_merge_sorts" => rotate_merge_sorts::options(&choice[1..]),
            "classic_merge_sorts" | _ => classic_merge_sorts::options(&choice[1..]),
        }
    }
}
