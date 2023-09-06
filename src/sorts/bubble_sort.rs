const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "bubble sort";

use crate::traits;
pub struct BubbleSort {}

impl traits::sort_traits::SortAlgo for BubbleSort {
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
    for ii in 0..arr.len() {
        for i in 1..arr.len() - ii {
            logger.log(SortLog::Cmp {
                name: 0,
                ind_a: i,
                ind_b: i - 1,
                result: arr[i] < arr[i - 1],
            });
            if arr[i] < arr[i - 1] {
                logger.log(SortLog::Swap {
                    name: 0,
                    ind_a: i - 1,
                    ind_b: i,
                });
                arr.swap(i, i - 1)
            }
        }
    }
}
