use std::marker::PhantomData;
use sort_logger::SortLogger;
use array_vis_bench_traits::SmallSort;
use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};

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

// Same N log N comparison/movement profile as the buffer-reusing variants.
impl<S: SmallSort> HasTimeBounds for NaiveMergeSort<S> {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}
// Naive recursion allocates a fresh L+R pair at every live frame, so all
// frames' buffers coexist: log N levels × N total per level → N log N peak
// aux (NOT the N1 of the single-buffer variants).
impl<S: SmallSort> HasSpace for NaiveMergeSort<S> {
    const SPACE: Complexity = Complexity::N_LOG_N;
}
impl<S: SmallSort> HasStability for NaiveMergeSort<S> {
    const STABLE: bool = true;
}

