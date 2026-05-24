use std::marker::PhantomData;

use crate::traits::complexity::Complexity;
use crate::traits::composable::{HasSpace, HasStability, HasTimeBounds};
use crate::traits::log_traits::SortLogger;
use crate::utils::rotation::{ReversalRotation, Rotation};

pub trait PartitionScheme {
    /// Display name used both in the `Partition` component slot and in
    /// the per-algorithm path the menu builds at startup.
    const NAME: &'static str;
    /// Partition `arr` with the pivot originally at `pivot_idx`.
    ///
    /// Returns `(left_end, right_start)`:
    /// - `arr[..left_end]` needs further sorting
    /// - `arr[right_start..]` needs further sorting
    /// - `arr[left_end..right_start]` is already placed
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize);
}

/// Lomuto partition (left-left single-pointer scan).
///
/// Moves the pivot to the end, scans left-to-right placing small elements
/// at the front, then swaps the pivot into its final position.
pub struct Lomuto;
combo_codegen::component!(Partition, Lomuto, "lomuto");

impl PartitionScheme for Lomuto {
    const NAME: &'static str = "lomuto";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        let len = arr.len();
        logger.swap(arr, pivot_idx, len - 1);
        let pivot = arr[len - 1];

        let mut small = 0;
        for i in 0..len - 1 {
            if logger.cmp_le_data(arr, i, pivot) {
                logger.swap(arr, i, small);
                small += 1;
            }
        }
        logger.swap(arr, small, len - 1);
        (small, small + 1)
    }
}

/// Hoare partition (left-right two-pointer scan).
///
/// Moves the pivot to the start, scans inward from both ends, then swaps the
/// pivot into its final position.
pub struct Hoare;
combo_codegen::component!(Partition, Hoare, "hoare");

impl PartitionScheme for Hoare {
    const NAME: &'static str = "hoare";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        logger.swap(arr, pivot_idx, 0);
        let pivot = arr[0];

        let mut left = 1;
        let mut right = arr.len() - 1;
        while left <= right {
            while left <= right && logger.cmp_le_data(arr, left, pivot) {
                left += 1;
            }
            while left <= right && logger.cmp_gt_data(arr, right, pivot) {
                right -= 1;
            }
            if left < right {
                logger.swap(arr, left, right);
                left += 1;
                right -= 1;
            }
        }
        logger.swap(arr, 0, right);
        (right, right + 1)
    }
}

/// Three-way partition (Dutch National Flag).
///
/// Splits into three regions: `< pivot`, `== pivot`, `> pivot`.
/// Equal elements are grouped in the middle and excluded from recursion.
pub struct ThreeWay;
combo_codegen::component!(Partition, ThreeWay, "three-way");

impl PartitionScheme for ThreeWay {
    const NAME: &'static str = "three-way";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        logger.swap(arr, pivot_idx, 0);
        let pivot = arr[0];

        let mut lt = 0; // end of "< pivot" region
        let mut i = 1; // scan pointer
        let mut gt = arr.len() - 1; // start of "> pivot" region

        while i <= gt {
            if logger.cmp_lt_data(arr, i, pivot) {
                logger.swap(arr, i, lt);
                lt += 1;
                i += 1;
            } else if logger.cmp_gt_data(arr, i, pivot) {
                logger.swap(arr, i, gt);
                if gt == 0 {
                    break;
                }
                gt -= 1;
                // don't advance i — swapped-in element not yet examined
            } else {
                i += 1; // == pivot
            }
        }
        (lt, gt + 1)
    }
}

/// Block partition (batched compare-then-swap).
///
/// Processes elements in fixed-size blocks: first classifies which elements
/// need to move (into offset buffers), then swaps them in a tight loop.
/// Reduces branch mispredictions compared to Lomuto/Hoare.
pub struct Block;
combo_codegen::component!(Partition, Block, "block");

impl PartitionScheme for Block {
    const NAME: &'static str = "block";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        let len = arr.len();
        logger.swap(arr, pivot_idx, len - 1);
        let pivot = arr[len - 1];

        const BLOCK: usize = 64;
        let mut offsets_l = [0usize; BLOCK];
        let mut offsets_r = [0usize; BLOCK];
        logger.log_aux_arr_u(&offsets_l);
        logger.log_aux_arr_u(&offsets_r);

        let mut left = 0;
        let mut right = len - 1; // pivot is at len-1

        while right - left > 2 * BLOCK {
            // Phase 1: classify left block
            let mut num_l = 0;
            for i in 0..BLOCK {
                if logger.cmp_gt_data(arr, left + i, pivot) {
                    logger.write_data_u(&mut offsets_l, num_l, i);
                    num_l += 1;
                }
            }
            // Phase 1: classify right block
            let mut num_r = 0;
            for i in 0..BLOCK {
                if logger.cmp_le_data(arr, right - 1 - i, pivot) {
                    logger.write_data_u(&mut offsets_r, num_r, i);
                    num_r += 1;
                }
            }
            // Phase 2: swap matching pairs
            let swaps = num_l.min(num_r);
            for s in 0..swaps {
                logger.swap(arr, left + offsets_l[s], right - 1 - offsets_r[s]);
            }
            // Advance the side that was fully consumed
            if num_l <= num_r {
                left += BLOCK;
            }
            if num_r <= num_l {
                right -= BLOCK;
            }
        }

        logger.free_aux_arr(&offsets_l);
        logger.free_aux_arr(&offsets_r);

        // Remainder: fall back to Lomuto for the leftover elements
        let mut small = left;
        for i in left..right {
            if logger.cmp_le_data(arr, i, pivot) {
                logger.swap(arr, i, small);
                small += 1;
            }
        }
        logger.swap(arr, small, len - 1);
        (small, small + 1)
    }
}

/// Moving-pivot partition.
///
/// Swaps the selected pivot to the start, then walks inward: elements smaller
/// than the current head extend the low region, larger elements are swapped to
/// the high end.
pub struct MovingPivot;
combo_codegen::component!(Partition, MovingPivot, "moving pivot");

impl PartitionScheme for MovingPivot {
    const NAME: &'static str = "moving pivot";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        logger.swap(arr, pivot_idx, 0);

        let mut low = 0;
        let mut high = arr.len() - 1;
        while low < high - 1 {
            if logger.cond_swap_le(arr, low + 1, low) {
                low += 1;
            } else {
                logger.swap(arr, low + 1, high);
                high -= 1;
            }
        }
        logger.cond_swap_lt(arr, high, low);
        (high, high)
    }
}

/// Moving-pivot partition, v2.

pub struct MovingPivotV2;
// WIP: partition algorithm currently fails correctness on reversed /
// arbitrary permutation inputs. The codegen registration line is left
// out of the cross-product until the algorithm is fixed (the build-time
// scanner is a plain text grep, so any in-comment reference here would
// still match — re-introduce a real registration call after the fix).

impl PartitionScheme for MovingPivotV2 {
    const NAME: &'static str = "moving pivot v2";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        _pivot_idx: usize,
    ) -> (usize, usize) {
        let mut pivot_ind = 0;
        let mut high = 0;

        while pivot_ind + 1 < high {
            let mut cmp_offset = 1;

            // count how many elements from pivot_ind are less than the pivot, starting with the next one 
            while pivot_ind + cmp_offset < arr.len() && logger.cmp_lt(arr, pivot_ind + cmp_offset, pivot_ind) {
                cmp_offset += 1;
            }
            // swap the block of smaller elements to the left of the pivot, stabaly by doing a series of adjacent swaps
            for i in 1..cmp_offset {
                logger.swap(arr, pivot_ind + i, pivot_ind + i - 1);
            }
            // move the pivot index to the end of the block of smaller elements -> 0..pivot_ind are smaller, then, pivotInd, then a block of larger elements followed by unknown elements
            pivot_ind += cmp_offset - 1;

            // count how many elements from pivot_ind are greater than the pivot, starting with the next one
            while pivot_ind + cmp_offset < arr.len() && logger.cmp_ge(arr, pivot_ind + cmp_offset, pivot_ind) {
                cmp_offset += 1;
            }
            // swap the block of larger elements to the right of the pivot, stabaly by doing rotation, then reverse the block of larger elements
            //todo, rotation arr[pivot_ind..], comp_offset
            //todo reverse arr[arr.len - cmp_offset..]

            // we now set arr to  arr[pivot_ind..arr.len - cmp_offset] and restart the loop.

        }
        // now we have a partition with the larger elements in preserved reversed order, the smaller elements are in preserved order, and the pivot is in the middle. we just need to reverse the larger elements to restore their original order and complete the partition.
        // todo reverse arr[pivot_ind + 1..]
        (pivot_ind, pivot_ind + 1)
    }
}




/// Stable moving-pivot partition, generic over a rotation strategy.
///
/// Each iteration scans the contiguous run of `< pivot` elements right of
/// the pivot, bubbles the pivot rightward through them (preserving their
/// relative order), then scans the next contiguous run of `>= pivot`
/// elements and rotates that block to the back of the still-unknown
/// region. The rotation `R` is order-preserving, so the unknown region
/// stays in original order across iterations — that's what keeps both
/// halves stable.
///
/// Each placed larger block is reversed in-place immediately after it
/// lands at the back. After the loop, a single global reverse over the
/// whole larger region flips both per-block order *and* cross-block order:
/// blocks come out in scan (= original) order, each block's contents in
/// original order. See the file's history for the worked-out trace.
///
/// Stable + in-place, but **O(N²) worst case** (the per-iteration
/// rotation walks the rest of the view). Aux space inherits from `R`.
pub struct MovingPivotV3<R: Rotation>(PhantomData<R>);

impl<R: Rotation> MovingPivotV3<R> {
    fn partition_impl<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        let n = arr.len();
        if n == 0 {
            return (0, 0);
        }
        logger.swap(arr, pivot_idx, 0);
        let pivot = arr[0];

        let scratch_size = R::scratch_size(n);
        let mut scratch: Vec<T> = if scratch_size > 0 {
            logger.create_aux_arr_t(scratch_size)
        } else {
            Vec::new()
        };

        let mut pivot_ind = 0usize;
        let mut high = n;

        while pivot_ind + 1 < high {
            // 1. Smaller-block scan: contiguous run of `< pivot` right of the pivot.
            let mut s = 0usize;
            while pivot_ind + 1 + s < high
                && logger.cmp_lt_data(arr, pivot_ind + 1 + s, pivot)
            {
                s += 1;
            }
            // 2. Bubble pivot rightward through the smaller block via adjacent
            //    swaps — each smaller element moves left by one, preserving
            //    their relative order.
            for i in 0..s {
                logger.swap(arr, pivot_ind + i, pivot_ind + i + 1);
            }
            pivot_ind += s;
            if pivot_ind + 1 >= high {
                break;
            }

            // 3. Larger-block scan: arr[pivot_ind+1] is not `< pivot` (else
            //    step 1 would have consumed it), so l >= 1.
            let mut l = 0usize;
            while pivot_ind + 1 + l < high
                && logger.cmp_ge_data(arr, pivot_ind + 1 + l, pivot)
            {
                l += 1;
            }

            // 4. Order-preserving rotation of the sub-slice
            //    arr[pivot_ind+1..high] = [larger (l)] [unknown (rest)].
            //    `R::rotate(slice, split)` puts `slice[split..]` at the front,
            //    so split = l makes the unknown the new prefix and pushes the
            //    larger block to the back.
            R::rotate(&mut arr[pivot_ind + 1..high], l, &mut scratch, logger);

            // 5. Reverse the just-placed block. Each placed block is left
            //    reversed at the back so the final global reverse fixes both
            //    per-block order and cross-iteration order in one pass.
            logger.reverse(&mut arr[high - l..high]);
            high -= l;
        }

        // Final reverse over the whole placed-larger region. Before: blocks
        // are stacked iter-K, iter-(K-1), …, iter-1 from front to back, each
        // individually reversed. After: blocks are iter-1, iter-2, …, iter-K
        // (scan order = original order), each in original order.
        if pivot_ind + 1 < n {
            logger.reverse(&mut arr[pivot_ind + 1..]);
        }

        if scratch_size > 0 {
            logger.free_aux_arr_t(&scratch);
        }

        (pivot_ind, pivot_ind + 1)
    }
}

// Per-rotation `PartitionScheme` impls. The trait's `const NAME` cannot
// reference the outer generic `R` (Rust limitation: const items inside
// generic impls don't capture outer type params), so each registered
// instance gets its own impl block with a hardcoded label. The algorithm
// body lives on the generic `partition_impl` above. The `component!` calls
// must stay at module scope — the codegen scanner is a text-grep that
// only sees top-level macro invocations, so they can't be hidden inside
// a macro_rules wrapper.
macro_rules! impl_moving_pivot_v3_trait {
    ($rot:ty, $name:literal) => {
        impl PartitionScheme for MovingPivotV3<$rot> {
            const NAME: &'static str = $name;
            fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
                arr: &mut [T],
                logger: &mut U,
                pivot_idx: usize,
            ) -> (usize, usize) {
                Self::partition_impl(arr, logger, pivot_idx)
            }
        }
    };
}

impl_moving_pivot_v3_trait!(ReversalRotation, "moving pivot v3<reversal>");
combo_codegen::component!(
    Partition,
    MovingPivotV3<ReversalRotation>,
    "moving pivot v3<reversal>"
);

// ── Composable annotations ──────────────────────────────────────────
//
// Every partition variant here walks the array once: O(N) time, O(1) aux
// space, no stable ordering preserved. The values are uniform across the
// five schemes — listed individually so adding a new partition that
// breaks one of these (e.g. a stable in-place partition, an aux-buffer
// scheme) is a one-line override.

macro_rules! impl_partition_annotations {
    ($ty:ty, $stable:expr) => {
        impl HasTimeBounds for $ty {
            const WORST: Complexity = Complexity::N1;
            const BEST: Complexity = Complexity::N1;
            const AVERAGE: Complexity = Complexity::N1;
        }
        impl HasSpace for $ty {
            const SPACE: Complexity = Complexity::CONST;
        }
        impl HasStability for $ty {
            const STABLE: bool = $stable;
        }
    };
}

impl_partition_annotations!(Lomuto, false);
impl_partition_annotations!(Hoare, false);
impl_partition_annotations!(ThreeWay, false);
impl_partition_annotations!(Block, false);
impl_partition_annotations!(MovingPivot, false);
impl_partition_annotations!(MovingPivotV2, false);

// MovingPivotV3<R>: stable + in-place, but the per-iteration rotation
// makes it O(N²) worst case (best case is still O(N) — one bubble pass
// when every element is < pivot). Space inherits from R: O(1) for
// in-place rotations, O(N) for AuxiliaryRotation.
impl<R: Rotation + HasSpace> HasTimeBounds for MovingPivotV3<R> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}
impl<R: Rotation + HasSpace> HasSpace for MovingPivotV3<R> {
    const SPACE: Complexity = R::SPACE;
}
impl<R: Rotation> HasStability for MovingPivotV3<R> {
    const STABLE: bool = true;
}
