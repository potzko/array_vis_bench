const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "quick surrender";

use crate::traits;
use rand::{rngs::ThreadRng, Rng};
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

fn quick_select<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    target: usize,
    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) {
    //println!("{}, {}", arr.len(), target);
    let piv = partition(arr, rng, logger);
    if piv == target {
        return;
    }
    if piv < target {
        quick_select(&mut arr[piv + 1..], target - piv - 1, rng, logger);
    } else {
        quick_select(&mut arr[0..piv], target, rng, logger)
    };
}

fn sort_helper<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    rng: &mut ThreadRng,
    logger: &mut U,
) {
    if arr.len() < 2 {
        return;
    }
    let mid = arr.len() / 2;
    quick_select(arr, mid, rng, logger);
    sort_helper(&mut arr[0..mid], rng, logger);
    sort_helper(&mut arr[mid + 1..], rng, logger);
}

fn sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    let mut rng = rand::thread_rng();
    sort_helper(arr, &mut rng, logger);
}

fn partition<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    rng: &mut rand::rngs::ThreadRng,
    logger: &mut U,
) -> usize {
    // Choose a random index between start and end - 1 as the pivot
    let pivot_index: usize = rng.gen_range(0..arr.len());
    //println!("{}, {}", arr.len(), pivot_index);

    // Swap the pivot with the last element
    logger.swap(arr, pivot_index, arr.len() - 1);

    let pivot = arr[arr.len() - 1];
    let mut small = 0;
    for i in 0..arr.len() - 1 {
        if logger.cmp_lt_data(arr, i, pivot) {
            logger.swap(arr, i, small);
            small += 1;
        }
    }
    logger.swap(arr, small, arr.len() - 1);
    small
}
