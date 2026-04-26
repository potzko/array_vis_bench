use crate::traits::log_traits::SortLogger;
use super::utils::{lower_bound, upper_bound, reverse};

const MIN_GALLOP: usize = 7;

combo_codegen::sort_family!(
    type = TimSort<{Gallop}>,
    uses = [
        "crate::sorts::merge_sorts::timsort::TimSort",
    ],
    Gallop: inline [("false", ""), ("true", "gallop")],
    name = "timsort",
    big_o = "O(N log N)",
    stable = true,
    direct_sort = true,
    path = ["merge sorts", "miscellaneous", "timsort", "{variant}"],
);

/// Timsort: adaptive, stable, hybrid merge/insertion sort.
///
/// - `GALLOP`: enable galloping mode — exponential skipping when one run
///   is consistently winning during a merge.
pub struct TimSort<const GALLOP: bool>;

impl<const GALLOP: bool> TimSort<GALLOP> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        if n < 64 {
            binary_insertion_sort(arr, 0, n, 1, logger);
            return;
        }

        let min_run = compute_minrun(n);
        let mut stack: Vec<(usize, usize)> = Vec::with_capacity(85); // (base, len)
        let mut lo = 0;

        while lo < n {
            let run_len = count_run(arr, lo, n, logger);
            let force = min_run.min(n - lo);
            let actual_len = if run_len < force {
                binary_insertion_sort(arr, lo, lo + force, lo + run_len, logger);
                force
            } else {
                run_len
            };

            stack.push((lo, actual_len));
            lo += actual_len;

            merge_collapse::<GALLOP, T, U>(arr, &mut stack, false, logger);
        }

        merge_collapse::<GALLOP, T, U>(arr, &mut stack, true, logger);
    }
}

// ---------------------------------------------------------------------------
// Minrun
// ---------------------------------------------------------------------------

fn compute_minrun(n: usize) -> usize {
    let mut r = 0usize;
    let mut n = n;
    while n >= 64 {
        r |= n & 1;
        n >>= 1;
    }
    n + r
}

// ---------------------------------------------------------------------------
// Run detection
// ---------------------------------------------------------------------------

/// Count the natural run starting at `lo`; reverse descending runs in-place.
fn count_run<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    n: usize,
    logger: &mut U,
) -> usize {
    if lo + 1 >= n {
        return 1;
    }
    let mut hi = lo + 1;
    if logger.cmp_gt(arr, lo, lo + 1) {
        // Strictly descending — extend, then reverse.
        hi += 1;
        while hi < n && logger.cmp_gt(arr, hi - 1, hi) {
            hi += 1;
        }
        reverse(&mut arr[lo..hi], logger);
    } else {
        // Non-decreasing — extend.
        hi += 1;
        while hi < n && !logger.cmp_gt(arr, hi - 1, hi) {
            hi += 1;
        }
    }
    hi - lo
}

// ---------------------------------------------------------------------------
// Binary insertion sort
// ---------------------------------------------------------------------------

/// Sort arr[lo..hi] assuming arr[lo..start] is already sorted.
fn binary_insertion_sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    start: usize,
    logger: &mut U,
) {
    let mut i = start.max(lo + 1);
    while i < hi {
        let pivot = arr[i];
        // Upper bound: first pos where arr[pos] > pivot (stable: equal elements keep order).
        let pos = upper_bound(arr, lo, i, pivot, logger);
        // Shift arr[pos..i] one slot right and insert pivot.
        logger.shift_insert(arr, i, pos, pivot);
        i += 1;
    }
}

// ---------------------------------------------------------------------------
// Merge-stack collapse
// ---------------------------------------------------------------------------

fn merge_collapse<const GALLOP: bool, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    stack: &mut Vec<(usize, usize)>,
    force_all: bool,
    logger: &mut U,
) {
    loop {
        let n = stack.len();
        if n <= 1 {
            break;
        }

        let (_, c) = stack[n - 1];
        let (_, b) = stack[n - 2];

        let merge_at = if n >= 3 {
            let (_, a) = stack[n - 3];
            if a <= b + c {
                // Merge the smaller adjacent pair for better balance.
                if a < c { n - 3 } else { n - 2 }
            } else if b <= c {
                n - 2
            } else if force_all {
                n - 2
            } else {
                break;
            }
        } else if b <= c || force_all {
            n - 2
        } else {
            break;
        };

        let (base_a, len_a) = stack[merge_at];
        let (_base_b, len_b) = stack[merge_at + 1];
        stack[merge_at] = (base_a, len_a + len_b);
        stack.remove(merge_at + 1);

        let mid = base_a + len_a;
        let hi = mid + len_b;
        merge::<GALLOP, T, U>(arr, base_a, mid, hi, logger);
    }
}

// ---------------------------------------------------------------------------
// Merge
// ---------------------------------------------------------------------------

fn merge<const GALLOP: bool, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    logger: &mut U,
) {
    if lo >= mid || mid >= hi {
        return;
    }
    // Early exit: already sorted.
    if !logger.cmp_gt(arr, mid - 1, mid) {
        return;
    }
    // Trim: skip elements already in their final position at both ends.
    let lo = lower_bound(arr, lo, mid, arr[mid], logger);
    let hi = upper_bound(arr, mid, hi, arr[mid - 1], logger);
    if lo >= mid || mid >= hi {
        return;
    }

    if mid - lo <= hi - mid {
        merge_lo::<GALLOP, T, U>(arr, lo, mid, hi, logger);
    } else {
        merge_hi::<GALLOP, T, U>(arr, lo, mid, hi, logger);
    }
}

// ---------------------------------------------------------------------------
// merge_lo: copy the shorter LEFT half; merge left-to-right.
// ---------------------------------------------------------------------------

fn merge_lo<const GALLOP: bool, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    logger: &mut U,
) {
    let left_len = mid - lo;
    let mut buf = logger.create_aux_arr_t(left_len);
    for i in 0..left_len {
        logger.write_accross(arr, lo + i, &mut buf, i);
    }

    let mut l = 0usize; // cursor into buf (left half)
    let mut r = mid;    // cursor into arr (right half)
    let mut dst = lo;   // write position in arr

    if !GALLOP {
        while l < left_len && r < hi {
            if logger.cmp_le_accross(&buf, l, arr, r) {
                logger.write_accross(&buf, l, arr, dst);
                l += 1;
            } else {
                let v = arr[r];
                logger.write_data(arr, dst, v);
                r += 1;
            }
            dst += 1;
        }
    } else {
        let mut min_gallop = MIN_GALLOP;
        'outer: loop {
            // ── One-at-a-time phase ─────────────────────────────────────
            let mut left_wins = 0usize;
            let mut right_wins = 0usize;
            loop {
                if l >= left_len || r >= hi {
                    break 'outer;
                }
                if logger.cmp_le_accross(&buf, l, arr, r) {
                    logger.write_accross(&buf, l, arr, dst);
                    l += 1;
                    dst += 1;
                    left_wins += 1;
                    right_wins = 0;
                } else {
                    let v = arr[r];
                    logger.write_data(arr, dst, v);
                    r += 1;
                    dst += 1;
                    right_wins += 1;
                    left_wins = 0;
                }
                if left_wins >= min_gallop || right_wins >= min_gallop {
                    break;
                }
            }

            // ── Gallop phase ─────────────────────────────────────────────
            loop {
                if l >= left_len || r >= hi {
                    break 'outer;
                }
                // How many buf elements starting at l are <= arr[r]?
                let key = arr[r];
                let count_l = upper_bound(&buf, l, left_len, key, logger) - l;
                for _ in 0..count_l {
                    logger.write_accross(&buf, l, arr, dst);
                    l += 1;
                    dst += 1;
                }
                if l >= left_len || r >= hi {
                    break 'outer;
                }
                // How many arr elements starting at r are < buf[l]?
                let key = buf[l];
                let count_r = lower_bound(arr, r, hi, key, logger) - r;
                for _ in 0..count_r {
                    let v = arr[r];
                    logger.write_data(arr, dst, v);
                    r += 1;
                    dst += 1;
                }
                if count_l < MIN_GALLOP && count_r < MIN_GALLOP {
                    min_gallop = (min_gallop + 1).min(MIN_GALLOP * 2);
                    break; // Back to one-at-a-time
                }
                min_gallop = min_gallop.saturating_sub(1).max(1);
            }
        }
    }

    // Copy any remaining left elements (right remainder is already in place).
    while l < left_len {
        logger.write_accross(&buf, l, arr, dst);
        l += 1;
        dst += 1;
    }
    logger.free_aux_arr_t(&buf);
}

// ---------------------------------------------------------------------------
// merge_hi: copy the shorter RIGHT half; merge right-to-left.
// ---------------------------------------------------------------------------

fn merge_hi<const GALLOP: bool, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    logger: &mut U,
) {
    let right_len = hi - mid;
    let mut buf = logger.create_aux_arr_t(right_len);
    for i in 0..right_len {
        logger.write_accross(arr, mid + i, &mut buf, i);
    }

    let mut l = mid;       // past-end of remaining left half in arr
    let mut r = right_len; // past-end of remaining right half in buf
    let mut dst = hi;      // past-end of write position

    if !GALLOP {
        while l > lo && r > 0 {
            dst -= 1;
            // buf[r-1] < arr[l-1]  →  arr[l-1] is larger, take it.
            if logger.cmp_lt_accross(&buf, r - 1, arr, l - 1) {
                l -= 1;
                let v = arr[l];
                logger.write_data(arr, dst, v);
            } else {
                r -= 1;
                logger.write_accross(&buf, r, arr, dst);
            }
        }
    } else {
        let mut min_gallop = MIN_GALLOP;
        'outer: loop {
            // ── One-at-a-time phase ─────────────────────────────────────
            let mut left_wins = 0usize;
            let mut right_wins = 0usize;
            loop {
                if l <= lo || r == 0 {
                    break 'outer;
                }
                dst -= 1;
                if logger.cmp_lt_accross(&buf, r - 1, arr, l - 1) {
                    l -= 1;
                    let v = arr[l];
                    logger.write_data(arr, dst, v);
                    left_wins += 1;
                    right_wins = 0;
                } else {
                    r -= 1;
                    logger.write_accross(&buf, r, arr, dst);
                    right_wins += 1;
                    left_wins = 0;
                }
                if left_wins >= min_gallop || right_wins >= min_gallop {
                    break;
                }
            }

            // ── Gallop phase (right-to-left) ─────────────────────────────
            loop {
                if l <= lo || r == 0 {
                    break 'outer;
                }
                // How many arr elements (from l-1 downward) are > buf[r-1]?
                let key = buf[r - 1];
                let pos_l = upper_bound(arr, lo, l, key, logger);
                let count_l = l - pos_l;
                for _ in 0..count_l {
                    l -= 1;
                    dst -= 1;
                    let v = arr[l];
                    logger.write_data(arr, dst, v);
                }
                if l <= lo || r == 0 {
                    break 'outer;
                }
                // How many buf elements (from r-1 downward) are >= arr[l-1]?
                let key = arr[l - 1];
                let pos_r = lower_bound(&buf, 0, r, key, logger);
                let count_r = r - pos_r;
                for _ in 0..count_r {
                    r -= 1;
                    dst -= 1;
                    logger.write_accross(&buf, r, arr, dst);
                }
                if count_l < MIN_GALLOP && count_r < MIN_GALLOP {
                    min_gallop = (min_gallop + 1).min(MIN_GALLOP * 2);
                    break;
                }
                min_gallop = min_gallop.saturating_sub(1).max(1);
            }
        }
    }

    // Copy any remaining right elements (left remainder is already in place).
    while r > 0 {
        r -= 1;
        dst -= 1;
        logger.write_accross(&buf, r, arr, dst);
    }
    logger.free_aux_arr_t(&buf);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::log_traits::NoOpLogger;

    fn check<const G: bool>(arr: &mut Vec<usize>) {
        let mut expected = arr.clone();
        expected.sort();
        TimSort::<G>::sort(arr, &mut NoOpLogger);
        assert_eq!(arr, &expected);
    }

    macro_rules! tim_tests {
        ($mod:ident, $g:expr) => {
            mod $mod {
                use super::*;
                #[test] fn empty()       { check::<$g>(&mut vec![]); }
                #[test] fn single()      { check::<$g>(&mut vec![1]); }
                #[test] fn two_rev()     { check::<$g>(&mut vec![2, 1]); }
                #[test] fn sorted_32()   { check::<$g>(&mut (0..32).collect()); }
                #[test] fn reversed_32() { check::<$g>(&mut (0..32usize).rev().collect()); }
                #[test] fn same_32()     { check::<$g>(&mut vec![42; 32]); }
                #[test] fn large_100()   { check::<$g>(&mut (0..100).map(|i| (i * 37 + 13) % 100).collect()); }
                #[test] fn large_1000()  { check::<$g>(&mut (0..1000).map(|i| (i * 37 + 13) % 1000).collect()); }
                #[test] fn descending()  { check::<$g>(&mut (0..200usize).rev().collect()); }
                #[test] fn all_same()    { check::<$g>(&mut vec![7; 200]); }
            }
        };
    }

    tim_tests!(no_gallop, false);
    tim_tests!(gallop, true);
}
