use crate::create_sort;

create_sort!(sort, "cycle sort", "O(N^2)", false);

fn sort<T: Ord + Copy, U: crate::traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let n = arr.len();
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
            std::mem::swap(&mut item, &mut arr[pos]);
            logger.write_data(arr, pos, item);
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
            if item != arr[pos] {
                std::mem::swap(&mut item, &mut arr[pos]);
                logger.write_data(arr, pos, item);
            }
        }
    }
}
