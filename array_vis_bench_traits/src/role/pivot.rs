//! `PivotSelector` and `DualPivotSelector` — pivot-selection roles.
//!
//! Leaf crates (`pivot_first`, `pivot_ninther`, …) implement
//! `PivotSelector` and live in their own tiny crates. Dual-pivot impls
//! (`CombinedSelector`, `NintherDualPivot`) currently still live in
//! `array_vis_bench`; they're closely coupled to the dual-pivot
//! quick-sort family and may move out in a later batch.
//!
//! `median_index` and `min_max_index` are shared helpers — multiple
//! pivots reach for them, so they live here next to the traits.

use sort_logger::SortLogger;

pub trait PivotSelector {
    /// Display name — used by the standalone partition / quick-select
    /// registration macros to spell out per-leaf path segments
    /// (`partitions/lomuto/<name>`).
    const NAME: &'static str;
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], logger: &mut U) -> usize;
}

/// Selects two pivot indices from a slice in a single call.
///
/// The returned indices `(p1, p2)` need not be ordered — the sort
/// algorithm normalises them. They *should* differ wherever possible.
pub trait DualPivotSelector {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> (usize, usize);
}

// ── Shared helpers ───────────────────────────────────────────────────────────

/// Return `(min_index, max_index)` among `arr[a]`, `arr[b]`, `arr[c]`
/// using 3 comparisons (optimal for both min and max of 3).
pub fn min_max_index<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &[T],
    logger: &mut U,
    a: usize,
    b: usize,
    c: usize,
) -> (usize, usize) {
    let (lo, hi) = if logger.cmp_ge(arr, a, b) { (b, a) } else { (a, b) };
    if logger.cmp_ge(arr, c, hi) {
        (lo, c)
    } else if logger.cmp_ge(arr, lo, c) {
        (c, hi)
    } else {
        (lo, hi)
    }
}

/// Return the index whose value is the median among `arr[a]`, `arr[b]`,
/// `arr[c]`.
pub fn median_index<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
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
            b
        } else if logger.cmp_le(arr, a, c) {
            c
        } else {
            a
        }
    } else if b_le_c {
        if logger.cmp_le(arr, a, c) {
            a
        } else {
            c
        }
    } else {
        b
    }
}
