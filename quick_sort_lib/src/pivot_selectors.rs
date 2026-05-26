use std::marker::PhantomData;
use sort_logger::SortLogger;

// Trait + the six simple impls + the median/min-max helpers all live in
// per-leaf crates now. `PivotSelector` is re-exported from the traits
// crate. Simple-pivot leaves (`pivot_first` etc.) are no longer
// re-exported from this module — depend on them directly. Keeping the
// re-exports here would force `quick_sort_lib` to drag every pivot leaf
// into every consumer, defeating the per-leaf split.
pub use array_vis_bench_traits::{DualPivotSelector, PivotSelector};
use array_vis_bench_traits::role::pivot::{median_index, min_max_index};

// NOTE: no `NAME` const on `DualPivotSelector`. `CombinedSelector<V1, V2>`'s
// natural name would be `concat(V1::NAME, V2::NAME)`, but consts inside an
// impl block can't capture generic parameters in current Rust. The
// dual-pivot registration macro takes the variant name as an explicit
// string instead.

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

// Composable annotations for the six simple pivots live in their leaf
// crates. CombinedSelector / NintherDualPivot are foreign-trait-on-local-
// type so they stay in this crate.
