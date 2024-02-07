const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "shell sort 2.25 shrink factor";

use crate::traits;
use std::marker::PhantomData;

pub struct SortImp<T: Ord + Copy, U: traits::log_traits::SortLogger<T>> {
    _markers: (PhantomData<T>, PhantomData<U>),
}

impl<T: Ord + Copy, U: traits::log_traits::SortLogger<T>> traits::sort_traits::SortAlgo<T, U>
    for SortImp<T, U>
{
    fn max_size() -> usize {
        MAX_SIZE
    }
    fn big_o() -> &'static str {
        BIG_O
    }
    fn sort(arr: &mut [T], logger: &mut U) {
        sort::<T, U>(arr, logger);
    }
    fn name() -> &'static str {
        NAME
    }
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    sort_rec(arr, 1, logger);

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

fn sort_rec<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    let len = arr.len() / jump;
    if len < 64 {
        insertion_sort_jump(arr, jump, logger);
        return;
    }
    let num = 32;
    for i in 0..num {
        sort_rec(&mut arr[jump * i..], jump * num, logger);
    }
    let num = 15;
    if len >= num * 16 {
        for i in 0..num {
            insertion_sort_jump(&mut arr[jump * i..], jump * num, logger);
        }
    }
    insertion_sort_jump(arr, jump, logger);
}

fn insertion_sort_jump<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    jump: usize,
    logger: &mut U,
) {
    for i in (0..arr.len()).step_by(jump) {
        let mut ind = i;
        while ind != 0 {
            if logger.cond_swap_lt(arr, ind, ind - jump) {
                ind -= jump;
            } else {
                break;
            }
        }
    }
}
