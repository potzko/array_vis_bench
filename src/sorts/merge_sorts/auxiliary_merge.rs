//! Auxiliary-array merges — the "normal" textbook merge that uses an
//! external scratch buffer, in contrast to the in-place rotation-based
//! merges in `rotation_merge.rs`.
//!
//! Two variants share a single [`AuxMerge`] trait so the standalone
//! registry can register them with the same macro shape used for the
//! rotation merges:
//!
//! - [`FullCopyAuxMerge`]: copy both halves into a length-N scratch
//!   buffer, then merge back into `arr`. Standard textbook form.
//! - [`HalfCopyAuxMerge`]: copy only the left half (length N/2);
//!   in-place merge with the right half (which stays put because the
//!   write head can never overtake the right read head). Half the aux
//!   space at the cost of a tiny amount of pointer arithmetic.

use crate::traits::complexity::Complexity;
use crate::traits::composable::{HasSpace, HasStability, HasTimeBounds};
use crate::traits::log_traits::SortLogger;

use super::utils::merge_inplace;

/// Strategy for merging `arr[..mid]` with `arr[mid..]` using an
/// auxiliary buffer. Same shape as [`super::rotation_merge::RotationMerge`]
/// — the standalone-merge registration treats them uniformly.
pub trait AuxMerge {
    /// Display name used in the per-algorithm path
    /// (`/merges/auxiliary/<name>`).
    const NAME: &'static str;
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], mid: usize, logger: &mut U);
}

/// Full-copy auxiliary merge.
///
/// Copy `arr` into a length-N aux buffer, then merge `aux[..mid]` and
/// `aux[mid..]` back into `arr`. O(N) aux space.
pub struct FullCopyAuxMerge;

impl HasTimeBounds for FullCopyAuxMerge {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for FullCopyAuxMerge {
    const SPACE: Complexity = Complexity::N1;
}
impl HasStability for FullCopyAuxMerge {
    const STABLE: bool = true;
}

impl AuxMerge for FullCopyAuxMerge {
    const NAME: &'static str = "full";
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        mid: usize,
        logger: &mut U,
    ) {
        let n = arr.len();
        if mid == 0 || mid == n {
            return;
        }
        let mut aux = logger.create_aux_arr_t(n);
        logger.copy_range(arr, 0, &mut aux, 0, n);
        let (al, ar) = aux.split_at(mid);
        merge_inplace(al, ar, arr, logger);
        logger.free_aux_arr_t(&aux);
    }
}

/// Half-copy auxiliary merge (Sedgewick-style).
///
/// Copy only `arr[..mid]` into an aux buffer; merge the aux against
/// `arr[mid..]` back into `arr` starting at position 0. Uses O(N/2)
/// aux space.
///
/// Correctness: write head `k` and right read head `j` both move
/// left-to-right with `k ≤ j` invariantly. The right half is read
/// before any of its cells are overwritten. When the left (aux) side
/// runs out, the remaining `arr[j..]` is already in place.
pub struct HalfCopyAuxMerge;

impl HasTimeBounds for HalfCopyAuxMerge {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for HalfCopyAuxMerge {
    const SPACE: Complexity = Complexity::N1;
}
impl HasStability for HalfCopyAuxMerge {
    const STABLE: bool = true;
}

impl AuxMerge for HalfCopyAuxMerge {
    const NAME: &'static str = "half";
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        mid: usize,
        logger: &mut U,
    ) {
        let n = arr.len();
        if mid == 0 || mid == n {
            return;
        }
        let mut aux = logger.create_aux_arr_t(mid);
        logger.copy_range(arr, 0, &mut aux, 0, mid);

        let mut i = 0;     // index into aux (left half)
        let mut j = mid;   // index into arr (right half, still in place)
        let mut k = 0;     // write position into arr
        while i < mid && j < n {
            if logger.cmp_le_accross(&aux, i, arr, j) {
                logger.write_accross(&aux, i, arr, k);
                i += 1;
            } else {
                logger.write(arr, k, j);
                j += 1;
            }
            k += 1;
        }
        while i < mid {
            logger.write_accross(&aux, i, arr, k);
            i += 1;
            k += 1;
        }
        // If `j < n` here, `arr[j..n]` is already in its final position.

        logger.free_aux_arr_t(&aux);
    }
}
