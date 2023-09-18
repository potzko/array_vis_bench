const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "insertion sort";

use crate::traits::{self, log_traits::SortLogger};
pub struct InsertionSort {}

impl traits::sort_traits::SortAlgo for InsertionSort {
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
        sort::<T, U>(arr, start, end, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn insertion_sort<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>(
    arr: &mut [T],
    start: usize,
    end: usize,
    logger: &mut U,
) {
    use crate::traits::log_traits::SortLog;
    for i in start + 1..end {
        let num = arr[i];
        let mut ind = i;
        if ind >= 1 {
            logger.log(SortLog::CmpOuterData {
                name: &arr as *const _ as usize,
                ind,
                data: num,
                result: arr[ind - 1] > num,
            })
        }
        while ind >= 1 && arr[ind - 1] > num {
            logger.log(SortLog::Write {
                name: &arr as *const _ as usize,
                ind_a: ind,
                ind_b: ind - 1,
            });
            arr[ind] = arr[ind - 1];
            ind -= 1;
            if ind >= 1 {
                logger.log(SortLog::CmpOuterData {
                    name: &arr as *const _ as usize,
                    ind,
                    data: num,
                    result: arr[ind - 1] > num,
                })
            };
        }
        logger.log(SortLog::WriteOutOfArr {
            name: &arr as *const _ as usize,
            ind: ind,
            data: num,
        });
        arr[ind] = num
    }
}

fn sort<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], start: usize, end: usize, logger: &mut U) {
    use crate::traits::log_traits::SortLog;
    logger.log(SortLog::CreateAuxArr {
        name: &arr as *const _ as usize,
        length: arr.len(),
    });

    insertion_sort(arr, start, end, logger)
}
