use std::marker::PhantomData;
use crate::traits::log_traits::SortLogger;
use crate::utils::small_sort::SmallSort;

combo_codegen::sort_family!(
    type = NaiveMergeSort<{SS}>,
    uses = [
        "crate::sorts::merge_sorts::naive::NaiveMergeSort",
        "crate::utils::small_sort::{NoSmallSort, Size1SmallSort, Size2SmallSort, InsertionSmallSort, NetworkSmallSort, Network16SmallSort}",
    ],
    SS: SmallSort,
    name = "naive merge sort",
    big_o = "O(N log N)",
    stable = true,
    direct_sort = true,
    path = ["merge sorts", "classic", "naive", "{variant}"],
);

/// Classic naive merge sort: allocates fresh left and right sub-arrays at
/// every recursion level, sorts each half, then merges back into the original.
///
/// Space: O(N log N) — unlike the single-buffer variants, every recursive
/// frame holds its own pair of aux arrays simultaneously.
pub struct NaiveMergeSort<S: SmallSort> {
    _phantom: PhantomData<S>,
}

impl<S: SmallSort> NaiveMergeSort<S> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        Self::sort_inner(arr, logger);
    }

    fn sort_inner<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        if S::THRESHOLD > 0 && n <= S::THRESHOLD {
            S::sort(arr, logger);
            return;
        }
        let mid = n / 2;
        let right_len = n - mid;

        // Copy each half into its own aux array.
        let mut left = logger.create_aux_arr_t(mid);
        logger.copy_range(arr, 0, &mut left, 0, mid);
        let mut right = logger.create_aux_arr_t(right_len);
        logger.copy_range(arr, mid, &mut right, 0, right_len);

        // Recursively sort both halves.
        Self::sort_inner(&mut left, logger);
        Self::sort_inner(&mut right, logger);

        // Merge left and right back into arr.
        let (mut l, mut r, mut i) = (0, 0, 0);
        while l < mid && r < right_len {
            if logger.cmp_le_accross(&left, l, &right, r) {
                logger.write_accross(&left, l, arr, i);
                l += 1;
            } else {
                logger.write_accross(&right, r, arr, i);
                r += 1;
            }
            i += 1;
        }
        while l < mid {
            logger.write_accross(&left, l, arr, i);
            l += 1;
            i += 1;
        }
        while r < right_len {
            logger.write_accross(&right, r, arr, i);
            r += 1;
            i += 1;
        }

        logger.free_aux_arr_t(&right);
        logger.free_aux_arr_t(&left);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::log_traits::NoOpLogger;
    use crate::utils::small_sort::{NoSmallSort, InsertionSmallSort};

    fn check<S: SmallSort>(arr: &mut Vec<usize>) {
        let mut expected = arr.clone();
        expected.sort();
        NaiveMergeSort::<S>::sort(arr, &mut NoOpLogger);
        assert_eq!(arr, &expected);
    }

    #[test] fn empty()       { check::<NoSmallSort>(&mut vec![]); }
    #[test] fn single()      { check::<NoSmallSort>(&mut vec![1]); }
    #[test] fn two_rev()     { check::<NoSmallSort>(&mut vec![2, 1]); }
    #[test] fn sorted_32()   { check::<NoSmallSort>(&mut (0..32).collect()); }
    #[test] fn reversed_32() { check::<NoSmallSort>(&mut (0..32usize).rev().collect()); }
    #[test] fn same_32()     { check::<NoSmallSort>(&mut vec![42; 32]); }
    #[test] fn large_100()   { check::<NoSmallSort>(&mut (0..100).map(|i| (i * 37 + 13) % 100).collect()); }
    #[test] fn threshold_32(){ check::<InsertionSmallSort<32>>(&mut (0..100).map(|i| (i * 37 + 13) % 100).collect()); }
}
