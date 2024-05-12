const MAX_SIZE: usize = 50000;
const BIG_O: &str = "O(N Log(N))";
const NAME: &str = "merge sort";

use super::utils::merge_inplace;
use crate::{sorts::merge_sorts::classic_merge_sorts::utils::copy_from_slice, traits};
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
    let tmp = merge_sort(arr, logger);
    for i in 0..arr.len() {
        logger.write_accross(&tmp, i, arr, i);
    }
    logger.free_aux_arr_t(&tmp)
}

fn merge_sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &[T],
    logger: &mut U,
) -> Vec<T> {
    if arr.len() < 2 {
        return logger.copy_aux_arr_t(&arr);
    }
    let left = merge_sort(&arr[..arr.len() / 2], logger);
    let right = merge_sort(&arr[arr.len() / 2..], logger);
    let mut ret = logger.create_aux_arr_t(arr.len());
    merge_inplace(&left, &right, &mut ret, logger);
    logger.free_aux_arr_t(&left);
    logger.free_aux_arr_t(&right);
    ret
}
