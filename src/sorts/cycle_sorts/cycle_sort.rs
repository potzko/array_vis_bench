use crate::create_sort;
use crate::traits::log_traits::SortLogger;

create_sort!(sort, "cycle sort", "O(N^2)", false);

fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    for cycle_start in 0..n - 1 {
        let mut item = arr[cycle_start];
        let mut pos = cycle_start;
        for i in cycle_start + 1..n {
            if logger.cmp_lt_data(arr, i, item) {
                pos += 1;
            }
        }
        if pos == cycle_start {
            continue;
        }
        while item == arr[pos] {
            pos += 1;
        }
        if pos != cycle_start {
            let displaced = arr[pos];
            logger.write_data(arr, pos, item);
            item = displaced;
        }
        while pos != cycle_start {
            pos = cycle_start;
            for i in cycle_start + 1..n {
                if logger.cmp_lt_data(arr, i, item) {
                    pos += 1;
                }
            }
            while item == arr[pos] {
                pos += 1;
            }
            let displaced = arr[pos];
            logger.write_data(arr, pos, item);
            item = displaced;
        }
    }
}

pub fn sort_dyn(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
    sort(arr, logger);
}
