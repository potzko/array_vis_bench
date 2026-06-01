use std::marker::PhantomData;
use sort_logger::SortLogger;

// Trait + the six simple impls + the median/min-max helpers all live in
// per-leaf crates now. `PivotSelector` is re-exported from the traits
// crate. Simple-pivot leaves (`pivot_first` etc.) are no longer
// re-exported from this module — depend on them directly. Keeping the
// re-exports here would force `quick_sort_lib` to drag every pivot leaf
// into every consumer, defeating the per-leaf split.
pub use array_vis_bench_traits::{DualPivotSelector, PivotInput, PivotSelector};
use array_vis_bench_traits::role::pivot::{median_index, min_max_index};
use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PivotQuality,
};

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

// Make CombinedSelector usable as a unified `PivotInput` with N = 2 so
// the same `QuickSort<P, V, SS>` machinery covers single- and
// dual-pivot variants.
impl<V1: PivotSelector, V2: PivotSelector> PivotInput for CombinedSelector<V1, V2> {
    const N: usize = 2;
    #[inline(always)]
    fn pick<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
        out: &mut [usize],
    ) {
        let (p1, p2) = <Self as DualPivotSelector>::select(arr, logger);
        out[0] = p1;
        out[1] = p2;
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

impl PivotInput for NintherDualPivot {
    const N: usize = 2;
    #[inline(always)]
    fn pick<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
        out: &mut [usize],
    ) {
        let (p1, p2) = <Self as DualPivotSelector>::select(arr, logger);
        out[0] = p1;
        out[1] = p2;
    }
}

// Composable annotations for the six simple pivots live in their leaf
// crates. CombinedSelector / NintherDualPivot are foreign-trait-on-local-
// type so they stay in this crate.

// ── Composable annotations for the dual-pivot types ─────────────────
//
// `CombinedSelector<V1, V2>` chains two single-pivot selections, so
// each axis's cost compounds: bounds sum over V1 + V2 and a degenerate
// V1 *or* V2 is enough to make the composed selector degenerate.
//
// `NintherDualPivot` does a constant amount of work (9 lookups + 3
// medians + 1 min-max) regardless of input size, and the sampling
// strategy makes it non-degenerate by construction.

impl<V1, V2> HasTimeBounds for CombinedSelector<V1, V2>
where
    V1: PivotSelector + HasTimeBounds,
    V2: PivotSelector + HasTimeBounds,
{
    const WORST: Complexity = Complexity::sum(V1::WORST, V2::WORST);
    const BEST: Complexity = Complexity::sum(V1::BEST, V2::BEST);
    const AVERAGE: Complexity = Complexity::sum(V1::AVERAGE, V2::AVERAGE);
}
impl<V1, V2> HasSpace for CombinedSelector<V1, V2>
where
    V1: PivotSelector + HasSpace,
    V2: PivotSelector + HasSpace,
{
    const SPACE: Complexity = Complexity::sum(V1::SPACE, V2::SPACE);
}
impl<V1, V2> HasStability for CombinedSelector<V1, V2>
where
    V1: PivotSelector + HasStability,
    V2: PivotSelector + HasStability,
{
    // Pivot *selection* doesn't reorder elements, so STABLE is academic
    // here; carry the AND for completeness.
    const STABLE: bool = V1::STABLE && V2::STABLE;
}
impl<V1, V2> PivotQuality for CombinedSelector<V1, V2>
where
    V1: PivotSelector + PivotQuality,
    V2: PivotSelector + PivotQuality,
{
    // Combined degenerates if either side does — the QuickSort worst
    // case picks O(N) recursion depth.
    const DEGENERATES: bool = V1::DEGENERATES || V2::DEGENERATES;
}

impl HasTimeBounds for NintherDualPivot {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for NintherDualPivot {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for NintherDualPivot {
    const STABLE: bool = true;
}
impl PivotQuality for NintherDualPivot {
    // Sample-based selector targets ~1/3 and ~2/3 quantiles. Worst
    // case still exists on adversarial inputs but is not the regular
    // O(N) recursion-depth degeneration the simple pivots have.
    const DEGENERATES: bool = false;
}
