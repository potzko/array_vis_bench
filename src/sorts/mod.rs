use crate::traits::log_traits::SortLogger;
use crate::utils::check_utils;

pub mod merge_sorts;
pub mod quick_sorts;

pub mod bubble_sorts;
pub mod circle_sorts;
pub mod comb_sorts;
pub mod cycle_sorts;
pub mod insertion_sorts;
pub mod rod_sorts;
pub mod shell_sorts;
pub mod heap_sort;
pub mod quick_heap_sort;
pub mod weak_heap_sort;

// Disconnected — pending port to sort_family! codegen:
// pub mod annotations;
// pub mod example_generic_sort;
// pub mod fun_sorts;
// pub mod heap_sort;

pub fn fn_sort(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
    choice: &[String],
) -> Vec<String> {
    let mut arr_original = arr.to_vec();
    let mut vals = if choice.is_empty() {
        vec![format!("no sort selected")]
    } else {
        match choice[0].as_str() {
            "merge_sorts" => merge_sorts::fn_sort(arr, logger, &choice[1..]),
            "bubble_sorts" => bubble_sorts::fn_sort(arr, logger, &choice[1..]),
            "circle_sorts" => circle_sorts::fn_sort(arr, logger, &choice[1..]),
            "comb_sorts" => comb_sorts::fn_sort(arr, logger, &choice[1..]),
            "cycle_sorts" => cycle_sorts::fn_sort(arr, logger, &choice[1..]),
            "insertion_sorts" => insertion_sorts::fn_sort(arr, logger, &choice[1..]),
            "rod_sorts" => rod_sorts::fn_sort(arr, logger, &choice[1..]),
            "shell_sorts" => shell_sorts::fn_sort(arr, logger, &choice[1..]),
            "heap_sorts" => heap_sort::fn_sort(arr, logger, &choice[1..]),
            "quick_heap_sorts" => quick_heap_sort::fn_sort(arr, logger, &choice[1..]),
            "weak_heap_sorts" => weak_heap_sort::fn_sort(arr, logger, &choice[1..]),
            _ => vec![format!("unknown sort family: {}", choice[0])],
        }
    };
    vals.push(format!(
        "stable_sorted: {}\n",
        check_utils::is_sorted_arr(arr, &mut arr_original)
    ));
    vals.push(format!("sorted: : {}\n", check_utils::is_sorted(arr)));
    vals
}

pub fn get_all_sorts() -> Vec<Vec<String>> {
    vec![]
}
