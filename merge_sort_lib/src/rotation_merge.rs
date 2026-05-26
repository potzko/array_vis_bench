use std::marker::PhantomData;
use array_vis_bench_traits::Complexity;
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use sort_logger::SortLogger;
use array_vis_bench_traits::Rotation;
use super::utils::{merge_rotation, lower_bound, upper_bound};

/// Strategy for merging two adjacent sorted slices in-place using rotation.
///
/// `scratch_size(n)` returns the size of an aux buffer the caller should
/// pre-allocate; `merge` then receives that buffer and forwards it to
/// every `R::rotate` invocation so the visualiser shows a single aux
/// array per sort run rather than one per rotation.
pub trait RotationMerge {
    /// Required scratch size for merging an array of length `n`. Delegates
    /// to the underlying rotation by default.
    fn scratch_size(n: usize) -> usize;

    /// Merge `arr[..mid]` and `arr[mid..]` in-place, using `scratch` as
    /// caller-owned aux memory passed down to the rotation.
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        mid: usize,
        scratch: &mut [T],
        logger: &mut U,
    );
}

// ---------------------------------------------------------------------------

/// Advance only from the left end: binary-search for how many right elements
/// are < arr[lo] before each rotation.
pub struct NaiveRotationMerge<R: Rotation> {
    _phantom: PhantomData<R>,
}

impl<R: Rotation> RotationMerge for NaiveRotationMerge<R> {
    #[inline]
    fn scratch_size(n: usize) -> usize {
        R::scratch_size(n)
    }

    #[inline(always)]
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        mid: usize,
        scratch: &mut [T],
        logger: &mut U,
    ) {
        merge_rotation::<R, T, U>(arr, mid, scratch, logger);
    }
}

// One rotation per inserted element × O(N) per rotation = O(N²). Best
// case (already merged) still walks the larger side once → O(N).
impl<R: Rotation> HasTimeBounds for NaiveRotationMerge<R> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}
impl<R: Rotation + HasSpace> HasSpace for NaiveRotationMerge<R> {
    const SPACE: Complexity = R::SPACE;
}
impl<R: Rotation> HasStability for NaiveRotationMerge<R> {
    /// Naive rotation merge preserves the order of equal keys (it
    /// binary-searches for `lower_bound` and rotates contiguous blocks).
    const STABLE: bool = true;
}

// ---------------------------------------------------------------------------

/// symMerge: O(N log N) data movements, O(N log² N) comparisons per merge.
///
/// Uses the global midpoint `p = (lo + hi) / 2` as the pivot element.
/// Finds where `arr[p]` belongs in the other half with ONE binary search,
/// does ONE rotation, then recurses on two independent sub-problems each of
/// size ≤ (hi - lo) / 2.  Recurrence T(N) = 2T(N/2) + O(N) per level gives
/// O(N log N) rotations and O(N log² N) comparisons total.
pub struct SmallerSideRotationMerge<R: Rotation> {
    _phantom: PhantomData<R>,
}

impl<R: Rotation> RotationMerge for SmallerSideRotationMerge<R> {
    #[inline]
    fn scratch_size(n: usize) -> usize {
        R::scratch_size(n)
    }

    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        mid: usize,
        scratch: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        if mid == 0 || mid == n || logger.cmp_le_accross(arr, mid - 1, arr, mid) {
            return;
        }
        sym_merge_r::<R, T, U>(arr, 0, mid, n, scratch, logger);
    }
}

// symMerge: O(N log N) data movements, O(N log² N) comparisons per merge.
// The comparison count is the dominant cost driving outer-loop timing.
impl<R: Rotation> HasTimeBounds for SmallerSideRotationMerge<R> {
    const WORST: Complexity = Complexity::N_LOG_SQUARED;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N_LOG_SQUARED;
}
impl<R: Rotation + HasSpace> HasSpace for SmallerSideRotationMerge<R> {
    const SPACE: Complexity = R::SPACE;
}
impl<R: Rotation> HasStability for SmallerSideRotationMerge<R> {
    /// symMerge picks pivots from the inside of each half via
    /// `lower_bound`/`upper_bound` so equal keys keep their original
    /// inter-half ordering.
    const STABLE: bool = true;
}

/// Recursive symMerge helper: merge `arr[lo..mid]` with `arr[mid..hi]`.
fn sym_merge_r<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    scratch: &mut [T],
    logger: &mut U,
) {
    if lo >= mid || mid >= hi {
        return;
    }
    // Base cases: one side has exactly 1 element — O(log N) binary search + O(N) rotation.
    if mid - lo == 1 {
        // Single left element: insert arr[lo] into the right half.
        let pos = lower_bound(arr, mid, hi, arr[lo], logger);
        R::rotate(&mut arr[lo..pos], 1, scratch, logger);
        return;
    }
    if hi - mid == 1 {
        // Single right element: insert arr[mid] into the left half.
        let pos = upper_bound(arr, lo, mid, arr[mid], logger);
        R::rotate(&mut arr[pos..hi], mid - pos, scratch, logger);
        return;
    }

    // Pick the pivot from the midpoint of the SHORTER half.
    // Using the global midpoint (lo+hi)/2 would land at `mid` when both halves are
    // equal length, making arr[p] = arr[mid] (minimum of right) and degenerating to
    // O(N²) behaviour.  Picking from inside the shorter half avoids this.
    let left_len = mid - lo;
    let right_len = hi - mid;

    if left_len <= right_len {
        // Left is shorter (or equal): pivot at left half's midpoint.
        // arr[p] is always strictly inside the left half (p < mid).
        let p = lo + left_len / 2;
        let q = lower_bound(arr, mid, hi, arr[p], logger);

        // Rotate arr[p..q] so that arr[mid..q] precedes arr[p..mid].
        // After rotation: arr[lo..p] | arr[mid..q] | arr[p..mid] | arr[q..hi]
        let new_mid = p + (q - mid);
        R::rotate(&mut arr[p..q], mid - p, scratch, logger);

        // Left sub-problem:  merge arr[lo..p]     with arr[p..new_mid]  (both ≤ arr[p])
        sym_merge_r::<R, T, U>(arr, lo, p, new_mid, scratch, logger);
        // Right sub-problem: merge arr[new_mid..q] with arr[q..hi]      (both ≥ arr[p])
        sym_merge_r::<R, T, U>(arr, new_mid, q, hi, scratch, logger);
    } else {
        // Right is shorter: pivot at right half's midpoint.
        // arr[p] is always strictly inside the right half (p >= mid).
        let p = mid + right_len / 2;
        let q = upper_bound(arr, lo, mid, arr[p], logger);

        // Degenerate guard: if q = mid then arr[p] >= every left element, so arr[p]
        // (and everything after it) is already in its final position.  Just merge
        // arr[lo..mid] with the shorter right prefix arr[mid..p] and return.
        if q == mid {
            sym_merge_r::<R, T, U>(arr, lo, mid, p, scratch, logger);
            return;
        }

        // Rotate arr[q..p+1] so that arr[mid..p+1] follows arr[q..mid].
        // After rotation: arr[lo..q] | arr[mid..p+1] | arr[q..mid] | arr[p+1..hi]
        let new_mid = q + (p + 1 - mid);
        R::rotate(&mut arr[q..p + 1], mid - q, scratch, logger);

        // Left sub-problem:  merge arr[lo..q]      with arr[q..new_mid]  (both ≤ arr[p])
        sym_merge_r::<R, T, U>(arr, lo, q, new_mid, scratch, logger);
        // Right sub-problem: merge arr[new_mid..p+1] with arr[p+1..hi]   (both ≥ arr[p])
        sym_merge_r::<R, T, U>(arr, new_mid, p + 1, hi, scratch, logger);
    }
}

