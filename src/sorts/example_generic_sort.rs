use crate::traits::*;
use sort_registry_macro::SortRegistry;

/// Example of a generic sort with a strategy parameter
pub struct GenericMergeSort<Strategy> {
    _phantom: std::marker::PhantomData<Strategy>,
}

impl<Strategy> GenericMergeSort<Strategy> {
    pub fn sort<T: Ord + Copy, U: log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U)
    where
        Strategy: MergeStrategy<T, U>,
    {
        if arr.len() <= 1 {
            return;
        }

        let mid = arr.len() / 2;
        Self::sort(&mut arr[..mid], logger);
        Self::sort(&mut arr[mid..], logger);
        Strategy::merge(arr, mid, logger);
    }
}

/// Trait for merge strategies
pub trait MergeStrategy<T: Ord + Copy, U: log_traits::SortLogger<T>> {
    fn merge(arr: &mut [T], mid: usize, logger: &mut U);
}

/// Classic merge strategy
#[derive(SortRegistry)]
pub struct ClassicMergeStrategy;

impl<T: Ord + Copy, U: log_traits::SortLogger<T>> MergeStrategy<T, U> for ClassicMergeStrategy {
    fn merge(arr: &mut [T], mid: usize, logger: &mut U) {
        let mut temp = arr.to_vec();
        let mut i = 0;
        let mut j = mid;
        let mut k = 0;

        while i < mid && j < arr.len() {
            if logger.cmp_le(arr, i, j) {
                temp[k] = arr[i];
                i += 1;
            } else {
                temp[k] = arr[j];
                j += 1;
            }
            k += 1;
        }

        while i < mid {
            temp[k] = arr[i];
            i += 1;
            k += 1;
        }

        while j < arr.len() {
            temp[k] = arr[j];
            j += 1;
            k += 1;
        }

        arr.copy_from_slice(&temp);
    }
}

/// In-place merge strategy
#[derive(SortRegistry)]
pub struct InPlaceMergeStrategy;

impl<T: Ord + Copy, U: log_traits::SortLogger<T>> MergeStrategy<T, U> for InPlaceMergeStrategy {
    fn merge(arr: &mut [T], mid: usize, logger: &mut U) {
        // In-place merge implementation
        let mut i = 0;
        let mut j = mid;

        while i < j && j < arr.len() {
            if logger.cmp_le(arr, i, j) {
                i += 1;
            } else {
                // Rotate the right portion
                let temp = arr[j];
                for k in (i..j).rev() {
                    logger.swap(arr, k, k + 1);
                }
                arr[i] = temp;
                i += 1;
                j += 1;
            }
        }
    }
}
