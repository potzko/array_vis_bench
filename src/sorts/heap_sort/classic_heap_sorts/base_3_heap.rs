const MAX_SIZE: usize = 100000;
const BIG_O: &str = "O(N*log(N))";
const NAME: &str = "heap sort";

const HEAP_SIZE: usize = 3;

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
    first_heapify(arr, logger);

    for i in (1..arr.len()).rev() {
        logger.swap(arr, 0, i);
        heapify(arr, 0, i, logger);
    }
}

fn first_heapify<T: Ord + Copy, U: traits::SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    for start in (0..arr.len() / HEAP_SIZE + HEAP_SIZE).rev() {
        heapify(arr, start, arr.len(), logger);
    }
}

fn heapify<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    use std::cmp::min;

    let mut ind = start;
    let mut max_child = start * HEAP_SIZE + 1;
    max_child = max_ind(arr, max_child, min(max_child + HEAP_SIZE, end), logger);

    while max_child < end {
        if logger.cond_swap_ge(arr, max_child, ind) {
            ind = max_child;
            max_child = max_ind(
                arr,
                ind * HEAP_SIZE + 1,
                min(ind * HEAP_SIZE + HEAP_SIZE + 1, end),
                logger,
            );
        } else {
            break;
        }
    }
}

fn max_ind<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) -> usize {
    let mut max = start;
    for i in start + 1..end {
        if logger.cmp_le(arr, max, i) {
            max = i
        }
    }
    max
}
