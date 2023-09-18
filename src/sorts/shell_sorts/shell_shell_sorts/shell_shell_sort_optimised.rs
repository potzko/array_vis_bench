const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "shell sort 2.25 shrink factor";

use crate::traits;
pub struct ShellSort {}

impl traits::sort_traits::SortAlgo for ShellSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
        arr: &mut [T],
        start: usize,
        end: usize,
        logger: &mut U,
    ) {
        sort_helper::<T, U>(arr, start, end, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn sort_helper<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    sort(arr, start, end, 1, logger);

    // Print TOTAL, COUNT and MAX values
    /*
    println!("Total: {}", *TOTAL.lock().unwrap());
    println!("Count: {}", *COUNT.lock().unwrap());
    println!(
        "avrage: {}",
        *TOTAL.lock().unwrap() / *COUNT.lock().unwrap()
    );
    */
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    jump: usize,
    logger: &mut U,
) {
    if end < start {
        return;
    }
    let len = (end - start) / jump;
    if len < 64 {
        insertion_sort_jump(arr, start, end, jump, logger);
        return;
    }
    let num = 32;
    for i in 0..num {
        sort(arr, start + jump * i, end, jump * num, logger);
    }
    let num = 15;
    if len >= num * 16 {
        for i in 0..num {
            insertion_sort_jump(arr, start + jump * i, end, jump * num, logger);
        }
    }
    insertion_sort_jump(arr, start, end, jump, logger);
}

fn insertion_sort_jump<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    jump: usize,
    logger: &mut U,
) {
    /*
    {
        let mut jumps_tracker = JUMPS_TRACKER.lock().unwrap();
        if jumps_tracker.insert(jump) {
            println!("{}", jump);
        }

        let len = (end - start) / jump;

        // Update TOTAL, COUNT and MAX
        {
            let mut total = TOTAL.lock().unwrap();
            *total += len;
        }
        {
            let mut count = COUNT.lock().unwrap();
            *count += 1;
        }
    }*/
    for i in (start..end).step_by(jump) {
        logger.log(traits::log_traits::SortLog::Mark(format!(
            "inserting value at index {}",
            i
        )));
        let mut ind = i;
        while ind != start {
            logger.log(traits::log_traits::SortLog::Cmp {
                name: &arr as *const _ as usize,
                ind_a: ind,
                ind_b: ind - jump,
                result: arr[ind] < arr[ind - jump],
            });
            if arr[ind] < arr[ind - jump] {
                logger.log(traits::log_traits::SortLog::Swap {
                    name: &arr as *const _ as usize,
                    ind_a: ind,
                    ind_b: ind - jump,
                });
                arr.swap(ind, ind - jump);
                ind -= jump;
            } else {
                break;
            }
        }
    }
}
