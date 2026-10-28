//! The bufferless merge, `bufferless_merge`: a stable in place merge of two
//! adjacent sorted runs that owns no storage. It is symMerge over triple
//! reversal rotations, so a merge of `n` elements costs `O(n)` comparisons and
//! `O(n log n)` moves through the logger. The `NoBuffer` and `NoScratch`
//! backings, and any backing whose buffer is too small for a run, merge with it.

use array_vis_bench_traits::role::RotationMerge;
use merge_sort_lib::SmallerSideRotationMerge;
use rotation_reversal::ReversalRotation;
use sort_logger::SortLogger;

/// Merge the sorted runs `arr[lo..mid]` and `arr[mid..hi]` in place, stably
/// (an equal key of the left run stays first).
pub fn bufferless_merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    logger: &mut U,
) {
    if lo >= mid || mid >= hi { return; }
    <SmallerSideRotationMerge<ReversalRotation> as RotationMerge>::merge(
        &mut arr[lo..hi],
        mid - lo,
        &mut [],
        logger,
    );
}
