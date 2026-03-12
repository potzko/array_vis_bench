use std::marker::PhantomData;
use crate::traits::log_traits::SortLogger;
use super::utils::{copy_across, merge_inplace};
use super::small_sort::SmallSort;

/// Bottom-up merge sort whose merge sequence mirrors top-down exactly.
///
/// Uses a Bresenham-style fixed-point stepping algorithm to compute the same
/// (lo, mid, hi) triples that a recursive top-down sort would produce, but
/// processes them level-by-level (breadth-first) without recursion.
///
/// For every n, this produces bit-for-bit identical comparison sequences to
/// `TopDownMergeSort<NoSmallSort, PING_PONG, EARLY_EXIT>`.
///
/// - `S`:          small-sort strategy applied to segments at the deepest level.
/// - `PING_PONG`:  alternate arr↔tmp each pass — no per-level copy-back.
/// - `EARLY_EXIT`: copy (instead of merge) segments that are already sorted.
pub struct TopDownMirrorMergeSort<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool> {
    _phantom: PhantomData<S>,
}

impl<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool>
    TopDownMirrorMergeSort<S, PING_PONG, EARLY_EXIT>
{
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }

        // Pre-sort initial chunks when a small-sort strategy is active.
        if S::THRESHOLD > 0 {
            let mut i = 0;
            while i < n {
                let end = (i + S::THRESHOLD).min(n);
                S::sort(&mut arr[i..end], logger);
                i += S::THRESHOLD;
            }
        }

        let mut tmp = logger.create_aux_arr_t(n);
        copy_across(arr, &mut tmp, logger);

        // d starts at the smallest power-of-2 >= n and halves each level.
        // At each level we emit the same merges as the recursive top-down at
        // that depth, visiting them left-to-right.
        // Skip the deepest levels already covered by the small-sort (if any).
        let mut d = n.next_power_of_two();
        if S::THRESHOLD > 0 {
            // Advance d down until each segment is larger than the threshold.
            while d / 2 >= S::THRESHOLD && d / 2 > 0 {
                d /= 2;
            }
        }

        // Track which buffer is currently the "source".
        // After each level the roles swap (ping-pong) or we copy back.
        // The pre-sorted chunks are in arr; tmp is a copy of arr (pre-sort).
        // Redo the copy so tmp matches arr after pre-sort.
        if S::THRESHOLD > 0 {
            copy_across(arr, &mut tmp, logger);
        }
        let mut src_is_arr = true;

        while d > 1 {
            if src_is_arr {
                Self::mirror_pass(arr, &mut tmp, n, d, logger);
            } else {
                Self::mirror_pass(&mut tmp, arr, n, d, logger);
            }
            if PING_PONG {
                src_is_arr = !src_is_arr;
            } else {
                // Copy-back: copy result (now in dst) back to src.
                if src_is_arr {
                    copy_across(&tmp[..n], arr, logger);
                } else {
                    copy_across(arr, &mut tmp[..n], logger);
                }
                // src_is_arr stays true (result is always back in arr).
            }
            d /= 2;
        }

        // If the final result landed in tmp (odd number of ping-pong levels),
        // copy it back to arr.
        if PING_PONG && !src_is_arr {
            copy_across(&tmp[..n], arr, logger);
        }

        logger.free_aux_arr_t(&tmp);
    }

    /// One level of the mirror algorithm.
    ///
    /// Bresenham stepping: each call to the inner loop advances by exactly
    /// `2 * (n/d)` elements on average, with sizes alternating between
    /// `floor(n/d)` and `ceil(n/d)` — exactly matching the recursive
    /// `mid = (lo + hi) / 2` split.
    fn mirror_pass<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        src: &[T],
        dst: &mut [T],
        n: usize,
        d: usize,
        logger: &mut U,
    ) {
        let mut i = 0usize;
        let mut dec = 0usize;

        while i < n {
            // Compute left-half end (m) and right-half end (b).
            dec += n;
            let m = (i + dec / d).min(n);
            dec %= d;
            dec += n;
            let b = (m + dec / d).min(n);
            dec %= d;

            if m > i && b > m {
                // Two non-empty halves — merge.
                if EARLY_EXIT && logger.cmp_le_accross(src, m - 1, src, m) {
                    copy_across(&src[i..b], &mut dst[i..b], logger);
                } else {
                    merge_inplace(&src[i..m], &src[m..b], &mut dst[i..b], logger);
                }
            } else {
                // Empty left half (segment size < 1 at this level) — copy to
                // keep dst in sync for the next level.
                copy_across(&src[i..b], &mut dst[i..b], logger);
            }

            i = b;
        }
    }
}
