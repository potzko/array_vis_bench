use std::marker::PhantomData;
use sort_logger::SortLogger;
use super::utils::{copy_across, merge_inplace};
use array_vis_bench_traits::SmallSort;
use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};

/// Bottom-up (iterative) merge sort.
///
/// - `S`:          small-sort strategy. `InsertionSmallSort<N>` pre-sorts all
///                 chunks of size N before merging; `NoSmallSort` starts from
///                 size-1 blocks (pure power-of-2 doubling).
/// - `PING_PONG`:  alternate arr↔tmp each pass — no per-segment copy-back.
/// - `EARLY_EXIT`: copy (instead of merge) segments that are already sorted.
pub struct BottomUpMergeSort<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool> {
    _phantom: PhantomData<S>,
}

impl<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool>
    BottomUpMergeSort<S, PING_PONG, EARLY_EXIT>
{
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }

        let gap0 = if S::THRESHOLD > 0 { S::THRESHOLD } else { 1 };

        // Pre-sort initial chunks.
        if S::THRESHOLD > 0 {
            let mut i = 0;
            while i < n {
                let end = (i + S::THRESHOLD).min(n);
                S::sort(&mut arr[i..end], logger);
                i += S::THRESHOLD;
            }
        }

        let mut tmp = logger.create_aux_arr_t(n);

        if PING_PONG {
            Self::sort_pp(arr, &mut tmp, n, gap0, logger);
        } else {
            Self::sort_cb(arr, &mut tmp, n, gap0, logger);
        }

        logger.free_aux_arr_t(&tmp);
    }

    fn sort_pp<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        tmp: &mut [T],
        n: usize,
        mut gap: usize,
        logger: &mut U,
    ) {
        copy_across(arr, tmp, logger);
        while gap < n {
            Self::merge_pass(arr, tmp, n, gap, logger);
            gap *= 2;
            if gap >= n {
                copy_across(tmp, arr, logger);
                break;
            }
            Self::merge_pass(tmp, arr, n, gap, logger);
            gap *= 2;
        }
    }

    fn sort_cb<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        tmp: &mut [T],
        n: usize,
        mut gap: usize,
        logger: &mut U,
    ) {
        while gap < n {
            let mut i = 0;
            while i < n {
                let mid = (i + gap).min(n);
                let end = (i + 2 * gap).min(n);
                if mid < n {
                    if EARLY_EXIT && logger.cmp_le_accross(arr, mid - 1, arr, mid) {
                        // already sorted — skip
                    } else {
                        merge_inplace(&arr[i..mid], &arr[mid..end], &mut tmp[i..end], logger);
                        copy_across(&tmp[i..end], &mut arr[i..end], logger);
                    }
                }
                i += 2 * gap;
            }
            gap *= 2;
        }
    }

    fn merge_pass<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        src: &[T],
        dst: &mut [T],
        n: usize,
        gap: usize,
        logger: &mut U,
    ) {
        let mut i = 0;
        while i < n {
            let mid = (i + gap).min(n);
            let end = (i + 2 * gap).min(n);
            if mid >= n {
                copy_across(&src[i..end], &mut dst[i..end], logger);
            } else if EARLY_EXIT && logger.cmp_le_accross(src, mid - 1, src, mid) {
                copy_across(&src[i..end], &mut dst[i..end], logger);
            } else {
                merge_inplace(&src[i..mid], &src[mid..end], &mut dst[i..end], logger);
            }
            i += 2 * gap;
        }
    }
}

// Iterative bottom-up merge: log N doubling passes, each touching all N
// elements → N log N in every case (EARLY_EXIT skips merges but still scans
// every pass boundary, so best stays N log N).
impl<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool> HasTimeBounds
    for BottomUpMergeSort<S, PING_PONG, EARLY_EXIT>
{
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}
// Single length-N scratch buffer.
impl<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool> HasSpace
    for BottomUpMergeSort<S, PING_PONG, EARLY_EXIT>
{
    const SPACE: Complexity = Complexity::N1;
}
impl<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool> HasStability
    for BottomUpMergeSort<S, PING_PONG, EARLY_EXIT>
{
    const STABLE: bool = true;
}
