use crate::traits::log_traits::SortLogger;
use crate::utils::rotation::Rotation;

/// Merge sorted slices `a` and `b` into `target`.
pub fn merge_inplace<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    a: &[T],
    b: &[T],
    target: &mut [T],
    logger: &mut U,
) {
    let mut ia = 0;
    let mut ib = 0;
    let mut ic = 0;
    while ia < a.len() && ib < b.len() {
        if logger.cmp_le_accross(a, ia, b, ib) {
            logger.write_accross(a, ia, target, ic);
            ia += 1;
        } else {
            logger.write_accross(b, ib, target, ic);
            ib += 1;
        }
        ic += 1;
    }
    if ia < a.len() {
        copy_across(&a[ia..], &mut target[ic..], logger);
    } else {
        copy_across(&b[ib..], &mut target[ic..], logger);
    }
}

/// Copy `src` into `dst`, logging each write as a cross-array write.
pub fn copy_across<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    src: &[T],
    dst: &mut [T],
    logger: &mut U,
) {
    logger.copy_range(src, 0, dst, 0, src.len().min(dst.len()));
}

// ---------------------------------------------------------------------------
// Binary search helpers
// ---------------------------------------------------------------------------

/// First index in `arr[start..end]` where `arr[i] >= pivot`.
#[inline]
pub fn lower_bound<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &[T],
    start: usize,
    end: usize,
    pivot: T,
    logger: &mut U,
) -> usize {
    let mut l = start;
    let mut r = end;
    while l < r {
        let m = l + (r - l) / 2;
        if logger.cmp_lt_data(arr, m, pivot) {
            l = m + 1;
        } else {
            r = m;
        }
    }
    l
}

/// First index in `arr[start..end]` where `arr[i] > pivot`.
#[inline]
pub fn upper_bound<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &[T],
    start: usize,
    end: usize,
    pivot: T,
    logger: &mut U,
) -> usize {
    let mut l = start;
    let mut r = end;
    while l < r {
        let m = l + (r - l) / 2;
        if logger.cmp_le_data(arr, m, pivot) {
            l = m + 1;
        } else {
            r = m;
        }
    }
    l
}

// ---------------------------------------------------------------------------

/// In-place merge of `arr[..mid]` and `arr[mid..]` using rotation `R`.
///
/// Advances left-only (naive), uses binary search to find how many right
/// elements to pull across before each rotation. `scratch` is forwarded
/// to every `R::rotate` call so the rotation can use a caller-owned aux
/// buffer (see [`Rotation::scratch_size`](crate::utils::rotation::Rotation::scratch_size)).
pub fn merge_rotation<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    mid: usize,
    scratch: &mut [T],
    logger: &mut U,
) {
    let n = arr.len();
    if mid == 0 || mid == n || logger.cmp_le_accross(arr, mid - 1, arr, mid) {
        return;
    }
    let mut lo = 0;
    let mut mid = mid;
    while lo < mid && mid < n {
        // Bulk-skip: advance lo past all left elements already <= arr[mid].
        let right_min = arr[mid];
        let skip = upper_bound(arr, lo, mid, right_min, logger);
        if skip > lo {
            lo = skip;
            continue;
        }
        // arr[lo] > arr[mid]: find k via binary search, then rotate.
        let pivot = arr[lo];
        let split = lower_bound(arr, mid, n, pivot, logger);
        let k = split - mid;
        R::rotate(&mut arr[lo..split], mid - lo, scratch, logger);
        lo += k;
        mid += k;
    }
}

/// Reverse a slice in-place.
pub use crate::utils::rotation::reverse;
