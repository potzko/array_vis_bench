//! Heap-aware partitions for the quickselect-based deep heapify.
//!
//! These partitions are parallel to the standard
//! [`crate::sorts::quick_sorts::partitions`] but operate in *logical heap
//! coordinates* (translated through `H::phys`) and use the heap's own
//! [`Compare`] to decide rootward-ness. That makes them direction-agnostic
//! — the same impl works for both `MaxForward` and `MinReverse` heaps —
//! at the cost of `H::phys` indirection on every access.
//!
//! Contract (matches [`crate::sorts::quick_sorts::partitions::PartitionScheme`]
//! but in logical space):
//!
//! - On entry: `arr` is the full array of length `n`; logical range
//!   `[lo, hi)` is the slice being partitioned; `pivot` is the logical
//!   index of the chosen pivot.
//! - On return: `(left_end, right_start)` are logical indices.
//!   - arr_logical[lo..left_end] are "more rootward" than the pivot
//!     (per `H::Compare`).
//!   - arr_logical[left_end..right_start] are placed (== pivot).
//!   - arr_logical[right_start..hi] are not-more-rootward than the pivot.
//!
//! All physical access goes through `H::phys(logical, n)` so swap events
//! the visualiser sees are at the real array positions.

use crate::compare::Compare;
use crate::heap::HeapLayout;
use sort_logger::SortLogger;

pub trait HeapPartition {
    fn partition<T, U, H>(
        arr: &mut [T],
        n: usize,
        lo: usize,
        hi: usize,
        pivot: usize,
        logger: &mut U,
    ) -> (usize, usize)
    where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        H: HeapLayout;
}

/// Left-left single-pointer partition — *Lomuto*-style (rootward-on-left scan).
pub struct LeftLeftPartition;

impl HeapPartition for LeftLeftPartition {
    fn partition<T, U, H>(
        arr: &mut [T],
        n: usize,
        lo: usize,
        hi: usize,
        pivot: usize,
        logger: &mut U,
    ) -> (usize, usize)
    where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        H: HeapLayout,
    {
        let last = hi - 1;
        let pivot_phys = H::phys(last, n);
        logger.swap(arr, H::phys(pivot, n), pivot_phys);

        let mut boundary = lo;
        for i in lo..last {
            let i_phys = H::phys(i, n);
            if <H::Compare as Compare>::comes_first(logger, arr, i_phys, pivot_phys) {
                logger.swap(arr, i_phys, H::phys(boundary, n));
                boundary += 1;
            }
        }
        logger.swap(arr, H::phys(boundary, n), pivot_phys);
        (boundary, boundary + 1)
    }
}

/// Left-right two-pointer partition — *Hoare*-style (scan inward from both ends).
pub struct LeftRightPartition;

impl HeapPartition for LeftRightPartition {
    fn partition<T, U, H>(
        arr: &mut [T],
        n: usize,
        lo: usize,
        hi: usize,
        pivot: usize,
        logger: &mut U,
    ) -> (usize, usize)
    where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        H: HeapLayout,
    {
        let pivot_phys_start = H::phys(lo, n);
        logger.swap(arr, H::phys(pivot, n), pivot_phys_start);
        // Pivot now sits at logical `lo`. Scan inward.
        let mut left = lo + 1;
        let mut right = hi - 1;
        while left <= right {
            while left <= right
                && <H::Compare as Compare>::comes_first_or_eq(
                    logger,
                    arr,
                    H::phys(left, n),
                    pivot_phys_start,
                )
            {
                left += 1;
            }
            while left <= right
                && <H::Compare as Compare>::comes_first(
                    logger,
                    arr,
                    pivot_phys_start,
                    H::phys(right, n),
                )
            {
                if right == 0 {
                    break;
                }
                right -= 1;
            }
            if left < right {
                logger.swap(arr, H::phys(left, n), H::phys(right, n));
                left += 1;
                if right == 0 {
                    break;
                }
                right -= 1;
            }
        }
        logger.swap(arr, pivot_phys_start, H::phys(right, n));
        (right, right + 1)
    }
}

/// Block partition — classify-then-swap in fixed-size blocks. Reduces
/// branch mispredictions on large slices; falls back to LeftLeftPartition-style
/// remainder loop.
pub struct Block;

impl HeapPartition for Block {
    fn partition<T, U, H>(
        arr: &mut [T],
        n: usize,
        lo: usize,
        hi: usize,
        pivot: usize,
        logger: &mut U,
    ) -> (usize, usize)
    where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        H: HeapLayout,
    {
        let last = hi - 1;
        let pivot_phys = H::phys(last, n);
        logger.swap(arr, H::phys(pivot, n), pivot_phys);

        const BLOCK: usize = 64;
        let mut offsets_l = [0usize; BLOCK];
        let mut offsets_r = [0usize; BLOCK];
        logger.log_aux_arr_u(&offsets_l);
        logger.log_aux_arr_u(&offsets_r);

        let mut left = lo;
        let mut right = last; // pivot sits at `last`

        while right - left > 2 * BLOCK {
            // Classify left block: which entries are NOT-more-rootward
            // than the pivot (need to move right)?
            let mut num_l = 0;
            for i in 0..BLOCK {
                let i_phys = H::phys(left + i, n);
                if !<H::Compare as Compare>::comes_first(logger, arr, i_phys, pivot_phys) {
                    logger.write_data_u(&mut offsets_l, num_l, i);
                    num_l += 1;
                }
            }
            // Classify right block: which entries ARE more-or-equally
            // rootward than the pivot (need to move left)?
            let mut num_r = 0;
            for i in 0..BLOCK {
                let r_phys = H::phys(right - 1 - i, n);
                if <H::Compare as Compare>::comes_first_or_eq(logger, arr, r_phys, pivot_phys) {
                    logger.write_data_u(&mut offsets_r, num_r, i);
                    num_r += 1;
                }
            }
            // Swap matching pairs.
            let swaps = num_l.min(num_r);
            for s in 0..swaps {
                logger.swap(
                    arr,
                    H::phys(left + offsets_l[s], n),
                    H::phys(right - 1 - offsets_r[s], n),
                );
            }
            if num_l <= num_r {
                left += BLOCK;
            }
            if num_r <= num_l {
                right -= BLOCK;
            }
        }

        logger.free_aux_arr(&offsets_l);
        logger.free_aux_arr(&offsets_r);

        // LeftLeftPartition-style remainder.
        let mut small = left;
        for i in left..right {
            let i_phys = H::phys(i, n);
            if <H::Compare as Compare>::comes_first_or_eq(logger, arr, i_phys, pivot_phys) {
                logger.swap(arr, i_phys, H::phys(small, n));
                small += 1;
            }
        }
        logger.swap(arr, H::phys(small, n), pivot_phys);
        (small, small + 1)
    }
}
