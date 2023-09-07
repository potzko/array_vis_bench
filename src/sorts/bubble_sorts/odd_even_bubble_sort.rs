const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "odd-even bubble sort";

use crate::traits;
pub struct OddEvenBubbleSort {}

impl traits::sort_traits::SortAlgo for OddEvenBubbleSort {
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
    if arr.len() < 2 {
        return;
    }
    let mut sorted: bool = false;
    while !sorted {
        sorted = true;
        logger.log(SortLog::Mark("parity = 0".to_string()));
        for ii in (2..arr.len()).step_by(2) {
            logger.log(SortLog::Cmp {
                name: 0,
                ind_a: ii - 1,
                ind_b: ii,
                result: arr[ii] < arr[ii - 1],
            });
            if arr[ii] < arr[ii - 1] {
                sorted = false;
                logger.log(SortLog::Swap {
                    name: 0,
                    ind_a: ii,
                    ind_b: ii - 1,
                });
                arr.swap(ii, ii - 1);
            }
        }
        logger.log(SortLog::Mark("parity = 1".to_string()));
        for ii in (1..arr.len()).step_by(2) {
            logger.log(SortLog::Cmp {
                name: 0,
                ind_a: ii,
                ind_b: ii - 1,
                result: arr[ii] < arr[ii - 1],
            });
            if arr[ii] < arr[ii - 1] {
                sorted = false;
                logger.log(SortLog::Swap {
                    name: 0,
                    ind_a: ii,
                    ind_b: ii - 1,
                });
                arr.swap(ii, ii - 1);
            }
        }
    }
}
