use crate::traits::SortLogger;
use crate::utils::check_utils;

pub mod bubble_sorts;
pub mod circle_sorts;
pub mod comb_sorts;
pub mod cycle_sorts;
pub mod fun_sorts;
pub mod heap_sort;
pub mod insertion_sorts;
pub mod merge_sorts;
pub mod quick_sorts;
pub mod shell_sorts;

pub fn fn_sort<T: Ord + Copy, U: SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
    choice: &[String],
) -> Vec<String> {
    let mut arr_original = arr.to_vec();
    let mut vals = if choice.is_empty() {
        circle_sorts::fn_sort(arr, logger, choice)
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "bubble_sorts" => bubble_sorts::fn_sort(arr, logger, &choice[1..]),
            "circle_sorts" => circle_sorts::fn_sort(arr, logger, &choice[1..]),
            "comb_sorts" => comb_sorts::fn_sort(arr, logger, &choice[1..]),
            "cycle_sorts" => cycle_sorts::fn_sort(arr, logger, &choice[1..]),
            "fun_sorts" => fun_sorts::fn_sort(arr, logger, &choice[1..]),
            "heap_sort" => heap_sort::fn_sort(arr, logger, &choice[1..]),
            "insertion_sorts" => insertion_sorts::fn_sort(arr, logger, &choice[1..]),
            "merge_sorts" => merge_sorts::fn_sort(arr, logger, &choice[1..]),
            "quick_sorts" => quick_sorts::fn_sort(arr, logger, &choice[1..]),
            "shell_sorts" | _ => shell_sorts::fn_sort(arr, logger, &choice[1..]),
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
        [
            "bubble_sorts",
            "circle_sorts",
            "comb_sorts",
            "cycle_sorts",
            "fun_sorts",
            "heap_sort",
            "insertion_sorts",
            "merge_sorts",
            "quick_sorts",
            "shell_sorts",
        ]
        .iter()
        .map(|i| i.to_string())
        .collect()
    } else {
        #[allow(clippy::wildcard_in_or_patterns)]
        match choice[0].as_str() {
            "bubble_sorts" => bubble_sorts::options(&choice[1..]),
            "comb_sorts" => comb_sorts::options(&choice[1..]),
            "circle_sorts" => circle_sorts::options(&choice[1..]),
            "cycle_sorts" => cycle_sorts::options(&choice[1..]),
            "fun_sorts" => fun_sorts::options(&choice[1..]),
            "heap_sort" => heap_sort::options(&choice[1..]),
            "insertion_sorts" => insertion_sorts::options(&choice[1..]),
            "merge_sorts" => merge_sorts::options(&choice[1..]),
            "quick_sorts" => quick_sorts::options(&choice[1..]),
            "shell_sorts" | _ => shell_sorts::options(&choice[1..]),
        }
    }
}

pub fn get_all_sorts() -> Vec<Vec<String>> {
    let mut queue: Vec<Vec<String>> = options(&[]).iter().map(|i| vec![i.clone()]).collect();
    let mut ret = vec![];
    println!("{:?}", queue);
    while let Some(path) = queue.pop() {
        let options = options(&path);
        if options.is_empty() {
            ret.push(path)
        } else {
            for option in options {
                let mut tmp = path.clone();
                tmp.push(option.clone());
                queue.push(tmp)
            }
        }
    }

    ret
}
