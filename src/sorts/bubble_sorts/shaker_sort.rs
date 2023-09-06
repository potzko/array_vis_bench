const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "shaker sort";

use crate::traits;
pub struct ShakerSort {}

impl traits::sort_traits::SortAlgo for ShakerSort {
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        BIG_O
    }
    fn sort<T: Ord, U: traits::log_traits::SortLogger>(arr: &mut [T], logger: &mut U) {
        sort::<T, U>(arr, logger);
    }
    fn name(&self) -> &str {
        NAME
    }
}

fn sort<T: Ord, U: traits::log_traits::SortLogger>(arr: &mut [T], logger: &mut U) {
    use traits::log_traits::SortLog;
    for ii in 0..arr.len() / 2 {
        for i in ii + 1..arr.len() - ii {
            logger.log(SortLog::Cmp {
                name: 0,
                ind_a: i,
                ind_b: i - 1,
                result: arr[i] < arr[i - 1],
            });
            if arr[i] < arr[i - 1] {
                logger.log(SortLog::Swap {
                    name: 0,
                    ind_a: i,
                    ind_b: i - 1,
                });
                arr.swap(i - 1, i);
            }
        }
        for i in (ii + 1..arr.len() - ii).rev() {
            logger.log(SortLog::Cmp {
                name: 0,
                ind_a: i,
                ind_b: i - 1,
                result: arr[i] < arr[i - 1],
            });
            if arr[i] < arr[i - 1] {
                logger.log(SortLog::Swap {
                    name: 0,
                    ind_a: i,
                    ind_b: i - 1,
                });
                arr.swap(i - 1, i);
            }
        }
    }
}
