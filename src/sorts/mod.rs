use crate::traits::log_traits::SortLogger;
use crate::utils::check_utils;

pub mod insertion_sorts;
pub mod shell_sorts;

// Sorts disconnected during refactor — see REFACTOR_PLAN.md.
// Uncomment a module and add its arm back to fn_sort/options below to restore.
// pub mod bubble_sorts;
// pub mod circle_sorts;
// pub mod comb_sorts;
// pub mod cycle_sorts;
// pub mod example_generic_sort;
// pub mod fun_sorts;
// pub mod heap_sort;
// pub mod merge_sorts;
// pub mod quick_sorts;

pub fn fn_sort(
    arr: &mut [usize],
    logger: &mut dyn SortLogger<usize>,
    choice: &[String],
) -> Vec<String> {
    let mut arr_original = arr.to_vec();
    let mut vals = if choice.is_empty() {
        insertion_sorts::fn_sort(arr, logger, choice)
    } else {
        match choice[0].as_str() {
            "shell_sorts" => shell_sorts::fn_sort(arr, logger, &choice[1..]),
            "insertion_sorts" | _ => insertion_sorts::fn_sort(arr, logger, &choice[1..]),
        }
    };
    vals.push(format!(
        "stable_sorted: {}\n",
        check_utils::is_sorted_arr(arr, &mut arr_original)
    ));
    vals.push(format!("sorted: : {}\n", check_utils::is_sorted(arr)));
    vals
}

pub fn options(choice: &[String]) -> Vec<String> {
    if choice.is_empty() {
        ["insertion_sorts", "shell_sorts"]
            .iter()
            .map(|i| i.to_string())
            .collect()
    } else {
        match choice[0].as_str() {
            "shell_sorts" => shell_sorts::options(&choice[1..]),
            "insertion_sorts" | _ => insertion_sorts::options(&choice[1..]),
        }
    }
}

pub fn get_all_sorts() -> Vec<Vec<String>> {
    // Sort discovery now goes through get_registered_sorts() (see REFACTOR_PLAN.md).
    // Kept for legacy compatibility.
    vec![]
}
