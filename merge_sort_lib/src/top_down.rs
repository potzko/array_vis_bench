use std::marker::PhantomData;
use sort_logger::SortLogger;
use super::utils::{copy_across, merge_inplace};
use array_vis_bench_traits::SmallSort;

/// Top-down (recursive) merge sort.
///
/// - `S`:          small-sort strategy (use `NoSmallSort` to recurse to size 1,
///                 or `InsertionSmallSort<N>` to cut off at N elements).
/// - `PING_PONG`:  swap arr/tmp roles each level — result lands in arr without
///                 a final copy-back.
/// - `EARLY_EXIT`: skip the merge when the two halves are already in order.
pub struct TopDownMergeSort<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool> {
    _phantom: PhantomData<S>,
}

impl<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool>
    TopDownMergeSort<S, PING_PONG, EARLY_EXIT>
{
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        let mut tmp = logger.create_aux_arr_t(arr.len());
        copy_across(arr, &mut tmp, logger);

        if PING_PONG {
            Self::sort_pp(arr, &mut tmp, logger);
        } else {
            Self::sort_cb(arr, &mut tmp, logger);
        }
        logger.free_aux_arr_t(&tmp);
    }

    /// Ping-pong: result lands in `out`. `scratch` holds the same data initially.
    fn sort_pp<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        out: &mut [T],
        scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = out.len();
        if n < 2 {
            return;
        }
        if S::THRESHOLD > 0 && n <= S::THRESHOLD {
            S::sort(out, logger);
            return;
        }
        let mid = n / 2;
        let (ol, or_) = out.split_at_mut(mid);
        let (sl, sr) = scratch.split_at_mut(mid);
        Self::sort_pp(sl, ol, logger);
        Self::sort_pp(sr, or_, logger);
        if EARLY_EXIT && logger.cmp_le_accross(sl, sl.len() - 1, sr, 0) {
            copy_across(sl, ol, logger);
            copy_across(sr, or_, logger);
            return;
        }
        merge_inplace(sl, sr, out, logger);
    }

    /// Copy-back: result stays in `arr`.
    fn sort_cb<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        tmp: &mut [T],
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
            let (al, ar) = arr.split_at_mut(mid);
            let (tl, tr) = tmp.split_at_mut(mid);
            Self::sort_cb(al, tl, logger);
            Self::sort_cb(ar, tr, logger);
        }
        if EARLY_EXIT {
            let (al, ar) = arr.split_at(mid);
            if logger.cmp_le_accross(al, mid - 1, ar, 0) {
                return;
            }
        }
        {
            let (al, ar) = arr.split_at(mid);
            merge_inplace(al, ar, tmp, logger);
        }
        copy_across(tmp, arr, logger);
    }
}
