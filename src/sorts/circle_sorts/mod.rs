use crate::{traits::SortAlgo, traits::SortLogger};

pub mod circle_sort_bottom_up;
pub mod circle_sort_bottom_up_increasing_size;
pub mod circle_sort_bottom_up_optimised;
pub mod circle_sort_recursive;
pub mod circle_sort_recursive_increasing_size;
pub mod circle_sort_recursive_optimised;
pub mod shaker_circle_sort;
pub mod shaker_circle_sort_bottom_up;
pub mod shaker_circle_sort_bottom_up_b;
pub mod stooge_circle_sort;
pub mod stooge_circle_sort_reversed;

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    if choice.is_empty() {
        let sort = circle_sort_recursive::CircleSort {};
        sort.sort(arr, logger);
        vec![format!("name: {}", sort.name())]
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "circle_sort_bottom_up" => {
                let sort = circle_sort_bottom_up::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "circle_sort_bottom_up_increasing_size" => {
                let sort = circle_sort_bottom_up_increasing_size::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "circle_sort_bottom_up_optimised" => {
                let sort = circle_sort_bottom_up_optimised::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "circle_sort_recursive_increasing_size" => {
                let sort = circle_sort_recursive_increasing_size::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "circle_sort_recursive_optimised" => {
                let sort = circle_sort_recursive_optimised::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shaker_circle_sort" => {
                let sort = shaker_circle_sort::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shaker_circle_sort_bottom_up" => {
                let sort = shaker_circle_sort_bottom_up::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "shaker_circle_sort_bottom_up_b" => {
                let sort = shaker_circle_sort_bottom_up_b::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "stooge_circle_sort" => {
                let sort = stooge_circle_sort::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "stooge_circle_sort_reversed" => {
                let sort = stooge_circle_sort_reversed::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
            "circle_sort_recursive" | _ => {
                let sort = circle_sort_recursive::CircleSort {};
                sort.sort(arr, logger);
                vec![format!("name: {}", sort.name())]
            }
        }
    }
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        [
            "circle_sort_bottom_up",
            "circle_sort_bottom_up_increasing_size",
            "circle_sort_bottom_up_optimised",
            "circle_sort_recursive",
            "circle_sort_recursive_increasing_size",
            "circle_sort_recursive_optimised",
            "shaker_circle_sort",
            "shaker_circle_sort_bottom_up",
            "shaker_circle_sort_bottom_up_b",
            "stooge_circle_sort",
            "stooge_circle_sort_reversed",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        vec![]
    }
}
