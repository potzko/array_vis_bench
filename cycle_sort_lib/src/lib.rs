use sort_logger::SortLogger;

pub struct CycleSort;

impl CycleSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
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
}

sort_registry_macro::sort_family! {
    type Sort = CycleSort;
    name        = "cycle sort";
    big_o       = "O(N^2)";
    stable      = false;
    direct_sort = true;
    path        = ["cycle sorts", "cycle sort"];
}
