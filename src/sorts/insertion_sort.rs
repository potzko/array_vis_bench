const MAX_SIZE: usize = 5000;
const BIG_O: &str = "O(N^2)";
const NAME: &str = "insertion sort";

use crate::traits;
pub struct InsertionSort {}

impl traits::sort_traits::SortAlgo for InsertionSort {
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
    for i in 1..arr.len() {
        logger.log(traits::log_traits::SortLog::Mark(format!(
            "inserting value at index {}",
            i
        )));
        let mut ind = i;
        while ind != 0 {
            logger.log(traits::log_traits::SortLog::Cmp {
                name: 0,
                ind_a: ind,
                ind_b: ind - 1,
                result: arr[ind] < arr[ind - 1],
            });
            if arr[ind] < arr[ind - 1] {
                logger.log(traits::log_traits::SortLog::Swap {
                    name: 0,
                    ind_a: ind,
                    ind_b: ind - 1,
                });
                arr.swap(ind, ind - 1);
                ind -= 1;
            } else {
                break;
            }
        }
    }
}
