use std::marker::PhantomData;
use crate::traits::log_traits::SortLogger;
use crate::utils::rotation::Rotation;
use super::utils::{merge_rotation, lower_bound, upper_bound};

/// Strategy for merging two adjacent sorted slices in-place using rotation.
pub trait RotationMerge {
    /// Merge `arr[..mid]` and `arr[mid..]` in-place.
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], mid: usize, logger: &mut U);
}

// ---------------------------------------------------------------------------

/// Advance only from the left end: binary-search for how many right elements
/// are < arr[lo] before each rotation.
pub struct NaiveRotationMerge<R: Rotation> {
    _phantom: PhantomData<R>,
}

impl<R: Rotation> RotationMerge for NaiveRotationMerge<R> {
    #[inline(always)]
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], mid: usize, logger: &mut U) {
        merge_rotation::<R, T, U>(arr, mid, logger);
    }
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
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], mid: usize, logger: &mut U) {
        let n = arr.len();
        if mid == 0 || mid == n || logger.cmp_le_accross(arr, mid - 1, arr, mid) {
            return;
        }
        sym_merge_r::<R, T, U>(arr, 0, mid, n, logger);
    }
}

/// Recursive symMerge helper: merge `arr[lo..mid]` with `arr[mid..hi]`.
fn sym_merge_r<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    logger: &mut U,
) {
    if lo >= mid || mid >= hi {
        return;
    }
    // Base cases: one side has exactly 1 element — O(log N) binary search + O(N) rotation.
    if mid - lo == 1 {
        // Single left element: insert arr[lo] into the right half.
        let pos = lower_bound(arr, mid, hi, arr[lo], logger);
        R::rotate(&mut arr[lo..pos], 1, logger);
        return;
    }
    if hi - mid == 1 {
        // Single right element: insert arr[mid] into the left half.
        let pos = upper_bound(arr, lo, mid, arr[mid], logger);
        R::rotate(&mut arr[pos..hi], mid - pos, logger);
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
        R::rotate(&mut arr[p..q], mid - p, logger);

        // Left sub-problem:  merge arr[lo..p]     with arr[p..new_mid]  (both ≤ arr[p])
        sym_merge_r::<R, T, U>(arr, lo, p, new_mid, logger);
        // Right sub-problem: merge arr[new_mid..q] with arr[q..hi]      (both ≥ arr[p])
        sym_merge_r::<R, T, U>(arr, new_mid, q, hi, logger);
    } else {
        // Right is shorter: pivot at right half's midpoint.
        // arr[p] is always strictly inside the right half (p >= mid).
        let p = mid + right_len / 2;
        let q = upper_bound(arr, lo, mid, arr[p], logger);

        // Degenerate guard: if q = mid then arr[p] >= every left element, so arr[p]
        // (and everything after it) is already in its final position.  Just merge
        // arr[lo..mid] with the shorter right prefix arr[mid..p] and return.
        if q == mid {
            sym_merge_r::<R, T, U>(arr, lo, mid, p, logger);
            return;
        }

        // Rotate arr[q..p+1] so that arr[mid..p+1] follows arr[q..mid].
        // After rotation: arr[lo..q] | arr[mid..p+1] | arr[q..mid] | arr[p+1..hi]
        let new_mid = q + (p + 1 - mid);
        R::rotate(&mut arr[q..p + 1], mid - q, logger);

        // Left sub-problem:  merge arr[lo..q]      with arr[q..new_mid]  (both ≤ arr[p])
        sym_merge_r::<R, T, U>(arr, lo, q, new_mid, logger);
        // Right sub-problem: merge arr[new_mid..p+1] with arr[p+1..hi]   (both ≥ arr[p])
        sym_merge_r::<R, T, U>(arr, new_mid, p + 1, hi, logger);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::log_traits::NoOpLogger;
    use crate::utils::rotation::{
        ReversalRotation, AuxiliaryRotation, BridgeRotation, ContrevRotation,
        TrinityRotation, GriesMillsRotation, GrailRotation, PistonRotation,
        HelixRotation, DrillRotation, JugglingRotation,
    };
    use crate::sorts::merge_sorts::rotation::{
        TopDownRotationMergeSort, BottomUpRotationMergeSort,
    };
    use crate::sorts::merge_sorts::small_sort::NoSmallSort;

    fn check_sort<S>(arr: &mut Vec<usize>, expected: &[usize])
    where
        S: for<'a> SortAlgo,
    {
        S::sort(arr, &mut NoOpLogger);
        assert_eq!(arr.as_slice(), expected, "sort failed");
    }

    trait SortAlgo {
        fn sort(arr: &mut [usize], logger: &mut NoOpLogger);
    }

    macro_rules! impl_algo {
        ($name:ident, $ty:ty) => {
            struct $name;
            impl SortAlgo for $name {
                fn sort(arr: &mut [usize], logger: &mut NoOpLogger) {
                    <$ty>::sort(arr, logger);
                }
            }
        };
    }

    impl_algo!(TdNaiveRev,  TopDownRotationMergeSort<NoSmallSort, NaiveRotationMerge<ReversalRotation>,        false>);
    impl_algo!(TdNaiveGm,   TopDownRotationMergeSort<NoSmallSort, NaiveRotationMerge<GriesMillsRotation>,      false>);
    impl_algo!(TdSsRev,     TopDownRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<ReversalRotation>,  false>);
    impl_algo!(TdSsGra,     TopDownRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<GrailRotation>,     false>);
    impl_algo!(BuNaiveRev,  BottomUpRotationMergeSort<NoSmallSort, NaiveRotationMerge<ReversalRotation>,       false>);
    impl_algo!(BuSsJug,     BottomUpRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<JugglingRotation>, false>);

    fn sorted(n: usize) -> Vec<usize>        { (0..n).collect() }
    fn reversed(n: usize) -> Vec<usize>      { (0..n).rev().collect() }
    fn all_same(n: usize) -> Vec<usize>      { vec![42; n] }
    fn alternating(n: usize) -> Vec<usize>   { (0..n).map(|i| if i % 2 == 0 { i } else { n - i }).collect() }

    macro_rules! sort_tests {
        ($name:ident, $algo:ty) => {
            mod $name {
                use super::*;
                #[test] fn empty()      { let mut a: Vec<usize> = vec![]; <$algo>::sort(&mut a, &mut NoOpLogger); assert!(a.is_empty()); }
                #[test] fn single()     { let mut a = vec![1]; <$algo>::sort(&mut a, &mut NoOpLogger); assert_eq!(a, [1]); }
                #[test] fn two_sorted() { let mut a = vec![1,2]; <$algo>::sort(&mut a, &mut NoOpLogger); assert_eq!(a, [1,2]); }
                #[test] fn two_rev()    { let mut a = vec![2,1]; <$algo>::sort(&mut a, &mut NoOpLogger); assert_eq!(a, [1,2]); }
                #[test] fn sorted_32()  { let mut a = sorted(32);     let e = sorted(32);     <$algo>::sort(&mut a, &mut NoOpLogger); assert_eq!(a, e); }
                #[test] fn reversed_32(){ let mut a = reversed(32);   let e = sorted(32);     <$algo>::sort(&mut a, &mut NoOpLogger); assert_eq!(a, e); }
                #[test] fn same_32()    { let mut a = all_same(32);   let e = all_same(32);   <$algo>::sort(&mut a, &mut NoOpLogger); assert_eq!(a, e); }
                #[test] fn alt_33()     { let mut a = alternating(33); let mut e = a.clone(); e.sort(); <$algo>::sort(&mut a, &mut NoOpLogger); assert_eq!(a, e); }
                #[test] fn large_100()  { let mut a: Vec<usize> = (0..100).map(|i| (i * 37 + 13) % 100).collect(); let mut e = a.clone(); e.sort(); <$algo>::sort(&mut a, &mut NoOpLogger); assert_eq!(a, e); }
            }
        };
    }

    sort_tests!(td_naive_rev, TopDownRotationMergeSort<NoSmallSort, NaiveRotationMerge<ReversalRotation>,        false>);
    sort_tests!(td_naive_gm,  TopDownRotationMergeSort<NoSmallSort, NaiveRotationMerge<GriesMillsRotation>,      false>);
    sort_tests!(td_ss_rev,    TopDownRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<ReversalRotation>,  false>);
    sort_tests!(td_ss_gra,    TopDownRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<GrailRotation>,     false>);
    sort_tests!(bu_naive_rev, BottomUpRotationMergeSort<NoSmallSort, NaiveRotationMerge<ReversalRotation>,       false>);
    sort_tests!(bu_ss_jug,    BottomUpRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<JugglingRotation>, false>);
    sort_tests!(td_ss_aux,    TopDownRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<AuxiliaryRotation>, false>);
    sort_tests!(td_ss_tri,    TopDownRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<TrinityRotation>,   false>);
    sort_tests!(td_ss_pis,    TopDownRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<PistonRotation>,    false>);
    sort_tests!(td_ss_hel,    TopDownRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<HelixRotation>,     false>);
    sort_tests!(td_ss_dri,    TopDownRotationMergeSort<NoSmallSort, SmallerSideRotationMerge<DrillRotation>,     false>);

    // ---------------------------------------------------------------------------
    // Stability tests: equal values must keep left-before-right order.
    //
    // We test sym_merge_r directly using (value, tag) pairs that compare only
    // by value (tag 0 = came from left half, 1 = came from right half).
    // After merging, for every run of equal values the tags must be
    // non-decreasing (all left-origin 0s before right-origin 1s).

    #[derive(Clone, Copy, Debug)]
    struct Lbl(u8, u8); // (value, tag: 0=left, 1=right)
    impl PartialEq for Lbl { fn eq(&self, o: &Self) -> bool { self.0 == o.0 } }
    impl Eq for Lbl {}
    impl PartialOrd for Lbl { fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(o)) } }
    impl Ord for Lbl { fn cmp(&self, o: &Self) -> std::cmp::Ordering { self.0.cmp(&o.0) } }

    fn check_stable_merge<R: Rotation>(left: &[u8], right: &[u8]) {
        let mid = left.len();
        let mut arr: Vec<Lbl> = left.iter().map(|&v| Lbl(v, 0))
            .chain(right.iter().map(|&v| Lbl(v, 1)))
            .collect();
        let hi = arr.len();
        sym_merge_r::<R, Lbl, NoOpLogger>(&mut arr, 0, mid, hi, &mut NoOpLogger);
        // verify sorted and stable
        for i in 1..arr.len() {
            assert!(arr[i-1].0 <= arr[i].0, "not sorted at {i}");
            if arr[i-1].0 == arr[i].0 {
                assert!(arr[i-1].1 <= arr[i].1,
                    "stability violated at {i}: left={:?} right={:?}", arr[i-1], arr[i]);
            }
        }
    }

    #[test]
    fn stable_equals_at_boundary() {
        // left-equal at split boundary, right-equal starts the right half
        check_stable_merge::<ReversalRotation>(&[1,2,3,3], &[3,4,5]);
    }
    #[test]
    fn stable_all_equal() {
        check_stable_merge::<ReversalRotation>(&[2,2,2], &[2,2,2]);
    }
    #[test]
    fn stable_interleaved_equals() {
        check_stable_merge::<ReversalRotation>(&[1,3,5,5,7], &[2,4,5,5,6,8]);
    }
    #[test]
    fn stable_right_singleton_equal() {
        check_stable_merge::<ReversalRotation>(&[2,2,3], &[2]);
    }
    #[test]
    fn stable_left_singleton_equal() {
        check_stable_merge::<ReversalRotation>(&[3], &[1,2,3,4]);
    }
    #[test]
    fn stable_grail() {
        check_stable_merge::<GrailRotation>(&[1,2,2,4], &[2,2,3,5]);
    }
    #[test]
    fn stable_juggling() {
        check_stable_merge::<JugglingRotation>(&[1,2,2,4], &[2,2,3,5]);
    }
}
