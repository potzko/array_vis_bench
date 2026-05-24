use crate::traits::log_traits::SortLogger;
use super::utils::{copy_across, merge_inplace, reverse};

combo_codegen::family!(
    type = NaturalMergeSort<{PP}, {EE}>,
    uses = [
        "crate::sorts::merge_sorts::natural::NaturalMergeSort",
    ],
    PP: inline [("false", ""), ("true", "ping-pong")],
    EE: inline [("false", ""), ("true", "early-exit")],
    name = "natural merge sort",
    big_o = "O(N log N)",
    space = "O(N)",
    stable = true,
    adaptive = true,
    direct_sort = true,
    path = ["merge sorts", "classic", "natural", "{variant}"],
);

/// Natural merge sort: detects maximal sorted runs and uses them as seeds.
///
/// Descending runs are reversed in-place. Best case O(n) on already-sorted input.
///
/// - `PING_PONG`:  alternate arr↔tmp each pass.
/// - `EARLY_EXIT`: skip the merge when adjacent runs are already in order.
pub struct NaturalMergeSort<const PING_PONG: bool, const EARLY_EXIT: bool>;

impl<const PING_PONG: bool, const EARLY_EXIT: bool> NaturalMergeSort<PING_PONG, EARLY_EXIT> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }

        let mut bounds = detect_runs(arr, logger);

        // Single run — already sorted.
        if bounds.len() == 2 {
            return;
        }

        let mut tmp = logger.create_aux_arr_t(n);
        copy_across(arr, &mut tmp, logger);

        let mut src_is_arr = true;

        while bounds.len() > 2 {
            let mut new_bounds: Vec<usize> = vec![0];

            if src_is_arr {
                merge_run_pass::<T, U, EARLY_EXIT>(arr, &mut tmp, &bounds, &mut new_bounds, logger);
            } else {
                merge_run_pass::<T, U, EARLY_EXIT>(&mut tmp, arr, &bounds, &mut new_bounds, logger);
            }

            bounds = new_bounds;

            if PING_PONG {
                src_is_arr = !src_is_arr;
            } else {
                // Copy result (in dst) back to arr.
                if src_is_arr {
                    copy_across(&tmp[..n], arr, logger);
                }
                // src_is_arr stays true.
            }
        }

        if PING_PONG && !src_is_arr {
            copy_across(&tmp[..n], arr, logger);
        }

        logger.free_aux_arr_t(&tmp);
    }
}

fn merge_run_pass<T: Ord + Copy, U: ?Sized + SortLogger<T>, const EARLY_EXIT: bool>(
    src: &[T],
    dst: &mut [T],
    bounds: &[usize],
    new_bounds: &mut Vec<usize>,
    logger: &mut U,
) {
    let mut bi = 0;
    while bi + 2 < bounds.len() {
        let lo = bounds[bi];
        let mid = bounds[bi + 1];
        let hi = bounds[bi + 2];
        if EARLY_EXIT && logger.cmp_le_accross(src, mid - 1, src, mid) {
            copy_across(&src[lo..hi], &mut dst[lo..hi], logger);
        } else {
            merge_inplace(&src[lo..mid], &src[mid..hi], &mut dst[lo..hi], logger);
        }
        new_bounds.push(hi);
        bi += 2;
    }
    // Odd run — carry over unchanged.
    if bi + 1 < bounds.len() {
        let lo = bounds[bi];
        let hi = bounds[bi + 1];
        copy_across(&src[lo..hi], &mut dst[lo..hi], logger);
        // Extend last boundary rather than adding a new one (it's already merged into
        // the final segment conceptually, but the loop will handle it next pass).
        new_bounds.push(hi);
    }
}

/// Scan `arr` for maximal ascending runs; reverse any strictly-descending runs.
/// Returns boundary indices `[0, end_of_run0, end_of_run1, ..., n]`.
fn detect_runs<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> Vec<usize> {
    let n = arr.len();
    let mut bounds = vec![0usize];
    let mut i = 0;

    while i < n {
        let start = i;
        i += 1;
        if i < n && logger.cmp_le_accross(arr, i, arr, i - 1) {
            // Strictly descending run — scan and reverse.
            while i < n && logger.cmp_le_accross(arr, i, arr, i - 1) {
                i += 1;
            }
            reverse(&mut arr[start..i], logger);
        } else {
            // Non-decreasing run — scan.
            while i < n && !logger.cmp_le_accross(arr, i, arr, i - 1) {
                i += 1;
            }
        }
        bounds.push(i);
    }

    bounds
}
