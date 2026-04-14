use std::marker::PhantomData;
use crate::traits::log_traits::SortLogger;

pub trait PivotSelector {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], logger: &mut U) -> usize;
}

pub struct FirstElement;
combo_codegen::component!(PivotSelector, FirstElement, "first");
pub struct MiddleElement;
combo_codegen::component!(PivotSelector, MiddleElement, "middle");
pub struct LastElement;
combo_codegen::component!(PivotSelector, LastElement, "last");
pub struct MedianOfThree;
combo_codegen::component!(PivotSelector, MedianOfThree, "median of 3");
pub struct MedianOfMedians;
combo_codegen::component!(PivotSelector, MedianOfMedians, "median of medians");
pub struct Ninther;
combo_codegen::component!(PivotSelector, Ninther, "ninther");

impl PivotSelector for FirstElement {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _arr: &[T],
        _logger: &mut U,
    ) -> usize {
        0
    }
}

impl PivotSelector for MiddleElement {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        _logger: &mut U,
    ) -> usize {
        arr.len() / 2
    }
}

impl PivotSelector for LastElement {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        _logger: &mut U,
    ) -> usize {
        arr.len() - 1
    }
}

impl PivotSelector for MedianOfThree {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> usize {
        median_index(arr, logger, 0, arr.len() / 2, arr.len() - 1)
    }
}

impl PivotSelector for MedianOfMedians {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> usize {
        let len = arr.len();
        if len < 5 {
            return len / 2;
        }
        let samples = [0, len / 4, len / 2, (3 * len) / 4, len - 1];
        let m1 = median_index(arr, logger, samples[0], samples[1], samples[2]);
        let m2 = median_index(arr, logger, samples[2], samples[3], samples[4]);
        median_index(arr, logger, m1, samples[2], m2)
    }
}

impl PivotSelector for Ninther {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> usize {
        let len = arr.len();
        if len < 9 {
            return median_index(arr, logger, 0, len / 2, len - 1);
        }
        // 9 evenly spaced samples, grouped into 3 triples
        let s = [
            0, len / 8, len / 4,              // Group A
            3 * len / 8, len / 2, 5 * len / 8, // Group B
            3 * len / 4, 7 * len / 8, len - 1, // Group C
        ];
        let m1 = median_index(arr, logger, s[0], s[1], s[2]);
        let m2 = median_index(arr, logger, s[3], s[4], s[5]);
        let m3 = median_index(arr, logger, s[6], s[7], s[8]);
        median_index(arr, logger, m1, m2, m3)
    }
}


/// Return `(min_index, max_index)` among arr[a], arr[b], arr[c] using 3
/// comparisons (optimal for both min and max of 3).
fn min_max_index<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &[T],
    logger: &mut U,
    a: usize,
    b: usize,
    c: usize,
) -> (usize, usize) {
    // After comparing a↔b: lo ≤ hi.
    let (lo, hi) = if logger.cmp_ge(arr, a, b) { (b, a) } else { (a, b) };
    // One more comparison against c settles both the new min and new max.
    if logger.cmp_ge(arr, c, hi) {
        (lo, c)   // c is the new max; lo is still the min
    } else if logger.cmp_ge(arr, lo, c) {
        (c, hi)   // c is the new min; hi is still the max
    } else {
        (lo, hi)  // c is in the middle; lo and hi are unchanged
    }
}

/// Return the index whose value is the median among arr[a], arr[b], arr[c].
fn median_index<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &[T],
    logger: &mut U,
    a: usize,
    b: usize,
    c: usize,
) -> usize {
    let a_le_b = logger.cmp_le(arr, a, b);
    let b_le_c = logger.cmp_le(arr, b, c);

    if a_le_b {
        if b_le_c {
            b // a <= b <= c
        } else if logger.cmp_le(arr, a, c) {
            c // a <= c < b
        } else {
            a // c < a <= b
        }
    } else if b_le_c {
        if logger.cmp_le(arr, a, c) {
            a // b < a <= c
        } else {
            c // b <= c < a
        }
    } else {
        b // c < b < a  →  median is b
    }
}

// ── DualPivotSelector ─────────────────────────────────────────────────────────

/// Selects two pivot indices from a slice in a single call.
///
/// The returned indices `(p1, p2)` need not be ordered — the sort algorithm
/// normalises them. They *should* differ wherever possible; see
/// [`CombinedSelector`] and [`NintherDualPivot`] for provided impls.
pub trait DualPivotSelector {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> (usize, usize);
}

// ── CombinedSelector ─────────────────────────────────────────────────────────

/// Wraps two independent [`PivotSelector`]s into a [`DualPivotSelector`].
///
/// `V1` selects the first pivot from the full array; `V2` selects the second
/// from `arr[1..]`, guaranteeing the two raw indices are never identical
/// (the second is always ≥ 1).
pub struct CombinedSelector<V1, V2>(PhantomData<(V1, V2)>);

impl<V1: PivotSelector, V2: PivotSelector> DualPivotSelector for CombinedSelector<V1, V2> {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> (usize, usize) {
        let p1 = V1::select(arr, logger);
        let p2 = if arr.len() > 1 { 1 + V2::select(&arr[1..], logger) } else { 0 };
        (p1, p2)
    }
}

// ── NintherDualPivot ─────────────────────────────────────────────────────────

/// A native dual-pivot selector that approximates the 1/3 and 2/3 quantiles in
/// one pass.
///
/// Samples 9 evenly-spaced positions from the full array, groups them into
/// three triples (lower, middle, upper third), and computes each triple's
/// median. The minimum and maximum of those three medians are returned as the
/// two pivots, naturally targeting the ~1/3 and ~2/3 quantiles and splitting the
/// array into three roughly equal parts.
pub struct NintherDualPivot;

impl DualPivotSelector for NintherDualPivot {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> (usize, usize) {
        let len = arr.len();
        if len < 9 {
            // Small array: use the two outer quartile positions.
            return (len / 4, 3 * len / 4);
        }
        let s = [
            0,           len / 8,     len / 4,      // Group A — lower third
            3 * len / 8, len / 2,     5 * len / 8,  // Group B — middle third
            3 * len / 4, 7 * len / 8, len - 1,      // Group C — upper third
        ];
        let m1 = median_index(arr, logger, s[0], s[1], s[2]);
        let m2 = median_index(arr, logger, s[3], s[4], s[5]);
        let m3 = median_index(arr, logger, s[6], s[7], s[8]);
        min_max_index(arr, logger, m1, m2, m3)
    }
}
