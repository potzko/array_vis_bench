const MAX_SIZE: usize = 1000;
const BIG_O: &str = "O(N^2)";

use crate::traits;
pub struct InsertionSort<T: Ord, U: traits::log_traits::SortLogger> {}

impl<T, U> traits::sort_traits::sort_algo<T, U> for InsertionSort<T, U>
where
    T: Ord,
    U: traits::log_traits::SortLogger,
{
    fn max_size(&self) -> usize {
        MAX_SIZE
    }
    fn big_o(&self) -> &str {
        &BIG_O
    }
    fn sort(&self, arr: &mut [T], logger: &mut U) {
        sort::<T, U>(arr, logger);
    }
}

fn sort<T: Ord, U: traits::log_traits::SortLogger>(arr: &mut [T], logger: &mut U) {
    for i in 1..arr.len() {
        logger.log(traits::log_traits::SortLog::Mark(format!(
            "inserting value at index {}",
            i
        )));
        let mut ind = i;
        for ii in ind..=0 {
            logger.log(traits::log_traits::SortLog::Cmp(
                0,
                ind,
                ind - 1,
                arr[ind] < arr[ind - 1],
            ));
            if arr[ind] < arr[ind - 1] {
                logger.log(traits::log_traits::SortLog::Swap(0, ind, ind - 1));
                arr.Swap(ind, ind - 1);
                ind -= 1;
            } else {
                break;
            }
        }
    }
}
