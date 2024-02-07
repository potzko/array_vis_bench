use crate::traits::{SortAlgo, SortLogger};

pub mod bubble_sort;
pub mod bubble_sort_recursive;
pub mod odd_even_bubble_sort;
pub mod shaker_sort;

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        type Sort<A, B> = bubble_sort::SortImp<A, B>;
        Sort::<T, U>::sort(arr, logger);
        vec![format!("name: {}", Sort::<T, U>::name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "bubble_sort_recursive" => {
                type Sort<A, B> = bubble_sort_recursive::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "odd_even_bubble_sort" => {
                type Sort<A, B> = odd_even_bubble_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "shaker_sort" => {
                type Sort<A, B> = shaker_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
            "bubble_sort" | _ => {
                type Sort<A, B> = bubble_sort::SortImp<A, B>;
                Sort::<T, U>::sort(arr, logger);
                vec![format!("name: {}", Sort::<T, U>::name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "bubble_sort",
            "bubble_sort_recursive",
            "shaker_sort",
            "bubble_sort",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
