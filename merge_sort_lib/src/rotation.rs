use std::marker::PhantomData;
use sort_logger::SortLogger;
use super::rotation_merge::RotationMerge;
use array_vis_bench_traits::SmallSort;

// Outer merge sort variants. Per-merge cost dominates the total — using
// NaiveRotationMerge (O(k²) per size-k merge) gives O(N²) overall;
// SmallerSideRotationMerge (O(k log² k) per merge) gives ≈ O(N log³ N)
// rounded to its dominant "N log² N" bucket for picker display.
/// Top-down (recursive) rotation merge sort.
///
/// Merges in-place using the `M` rotation strategy — no auxiliary array.
///
/// - `S`:          small-sort strategy.
/// - `M`:          rotation merge strategy (`NaiveRotation` or `SmallerSideRotation`).
/// - `EARLY_EXIT`: skip the merge when the two halves are already in order.
pub struct TopDownRotationMergeSort<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool> {
    _phantom: PhantomData<(S, M)>,
}

impl<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool>
    TopDownRotationMergeSort<S, M, EARLY_EXIT>
{
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let scratch_size = M::scratch_size(n);
        if scratch_size == 0 {
            Self::sort_rec(arr, &mut [], logger);
        } else {
            let mut scratch = logger.create_aux_arr_t(scratch_size);
            Self::sort_rec(arr, &mut scratch, logger);
            logger.free_aux_arr_t(&scratch);
        }
    }

    fn sort_rec<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        if S::THRESHOLD > 0 && n <= S::THRESHOLD {
            S::sort(arr, logger);
            return;
        }
        let mid = n / 2;
        {
            let (l, r) = arr.split_at_mut(mid);
            Self::sort_rec(l, scratch, logger);
            Self::sort_rec(r, scratch, logger);
        }
        if EARLY_EXIT && logger.cmp_le_accross(arr, mid - 1, arr, mid) {
            return;
        }
        M::merge(arr, mid, scratch, logger);
    }
}

/// Bottom-up (iterative) rotation merge sort.
///
/// - `S`:          small-sort strategy.
/// - `M`:          rotation merge strategy.
/// - `EARLY_EXIT`: skip the merge when a segment pair is already sorted.
pub struct BottomUpRotationMergeSort<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool> {
    _phantom: PhantomData<(S, M)>,
}

impl<S: SmallSort, M: RotationMerge, const EARLY_EXIT: bool>
    BottomUpRotationMergeSort<S, M, EARLY_EXIT>
{
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        let scratch_size = M::scratch_size(n);
        if scratch_size == 0 {
            Self::sort_inner(arr, &mut [], logger);
        } else {
            let mut scratch = logger.create_aux_arr_t(scratch_size);
            Self::sort_inner(arr, &mut scratch, logger);
            logger.free_aux_arr_t(&scratch);
        }
    }

    fn sort_inner<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        let gap0 = if S::THRESHOLD > 0 { S::THRESHOLD } else { 1 };

        if S::THRESHOLD > 0 {
            let mut i = 0;
            while i < n {
                let end = (i + S::THRESHOLD).min(n);
                S::sort(&mut arr[i..end], logger);
                i += S::THRESHOLD;
            }
        }

        let mut gap = gap0;
        while gap < n {
            let mut i = 0;
            while i < n {
                let mid = (i + gap).min(n);
                let end = (i + 2 * gap).min(n);
                if mid < end {
                    if EARLY_EXIT && logger.cmp_le_accross(arr, mid - 1, arr, mid) {
                        // already sorted — skip
                    } else {
                        M::merge(&mut arr[i..end], mid - i, scratch, logger);
                    }
                }
                i += 2 * gap;
            }
            gap *= 2;
        }
    }
}
