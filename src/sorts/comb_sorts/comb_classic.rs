use crate::create_sort;

create_sort!(sort, "comb sort classic", "O(N^2)", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut gap = arr.len();
    let mut swapped = true;
    while gap > 1 || swapped {
        if gap > 1 {
            gap = (gap as f64 / 1.3) as usize;
        }
        swapped = false;
        for i in 0..arr.len() - gap {
            if logger.cond_swap_lt(arr, i + gap, i) {
                swapped = true;
            }
        }
    }
}
