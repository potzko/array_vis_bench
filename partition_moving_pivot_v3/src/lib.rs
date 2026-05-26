//! Stable in-place moving-pivot partition (Phase 3 design — keeps the
//! smaller block stable through adjacent swaps, rotates the larger block
//! to the back of the still-unknown region using an order-preserving
//! `Rotation`).
//!
//! O(N²) worst case (per-iteration rotation walks the rest of the view),
//! best case O(N). Aux space inherits from `R`.

use std::marker::PhantomData;

use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionVisitor, Rotation,
};
use rotation_reversal::ReversalRotation;
use sort_logger::SortLogger;

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
            // 1. Smaller-block scan: contiguous run of `< pivot`.
            let mut s = 0usize;
            while pivot_ind + 1 + s < high
                && logger.cmp_lt_data(arr, pivot_ind + 1 + s, pivot)
            {
                s += 1;
            }
            // 2. Bubble pivot rightward via adjacent swaps — preserves
            //    the smaller block's relative order.
            for i in 0..s {
                logger.swap(arr, pivot_ind + i, pivot_ind + i + 1);
            }
            pivot_ind += s;
            if pivot_ind + 1 >= high {
                break;
            }

            // 3. Larger-block scan: arr[pivot_ind+1] is not `< pivot`
            //    (step 1 would have consumed it), so l >= 1.
            let mut l = 0usize;
            while pivot_ind + 1 + l < high
                && logger.cmp_ge_data(arr, pivot_ind + 1 + l, pivot)
            {
                l += 1;
            }

            // 4. Order-preserving rotation pushes the larger block to
            //    the back of the unknown region.
            R::rotate(&mut arr[pivot_ind + 1..high], l, &mut scratch, logger);

            // 5. Reverse just-placed block so the final global reverse
            //    fixes both per-block order and cross-iteration order.
            logger.reverse(&mut arr[high - l..high]);
            high -= l;
        }

        // Final reverse over the whole placed-larger region.
        if pivot_ind + 1 < n {
            logger.reverse(&mut arr[pivot_ind + 1..]);
        }

        if scratch_size > 0 {
            logger.free_aux_arr_t(&scratch);
        }

        (pivot_ind, pivot_ind + 1)
    }
}

// Per-rotation `PartitionScheme` impls. The trait's `const NAME` can't
// reference the outer generic `R` (Rust limitation: const items inside
// generic impls don't capture outer type params), so each specialisation
// gets its own impl block with a hardcoded label.
impl PartitionScheme for MovingPivotV3<ReversalRotation> {
    const NAME: &'static str = "moving pivot v3<reversal>";
    const N_PIVOTS: usize = 1;
    #[inline]
    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        pivots: &[usize],
        visitor: &mut V,
    ) where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor,
    {
        let len = arr.len();
        let (left_end, right_start) = Self::partition_impl(arr, logger, pivots[0]);
        visitor.unsorted(0..left_end);
        visitor.unsorted(right_start..len);
    }
}

// MovingPivotV3<R>: stable + in-place, O(N²) worst, best O(N). Space
// inherits from R: O(1) for in-place rotations, O(N) for AuxiliaryRotation.
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
