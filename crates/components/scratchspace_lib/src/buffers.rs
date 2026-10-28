//! The two `Auxiliary` backings of a block merge sort. `InArrayBuffer` carves its
//! buffer out of the array, so it allocates nothing and its values are swapped
//! with real elements. `FixedBuffer` is a small external buffer, a free overwrite
//! but too small to hold the blocks of a large merge.

use array_vis_bench_traits::role::Auxiliary;
use array_vis_bench_traits::Complexity;

/// `isqrt(n)` distinct elements of the array serve as the merge buffer, `O(1)`
/// space. Writing to it is a swap, and the buffer is sorted back into the array
/// when the sort ends. It holds a block of any merge, so merges stay linear.
pub struct InArrayBuffer;
impl Auxiliary for InArrayBuffer {
    const NAME: &'static str = "in array";
    const MERGE_WORST: Complexity = Complexity::N1;
    const MERGE_SPACE: Complexity = Complexity::CONST;

    fn scratch_size(_n: usize) -> usize { 0 }

    fn internal_size(n: usize) -> usize { n.isqrt() }
}

/// A fixed external buffer of `CAPACITY` cells, `O(1)` space. A merge whose run
/// fits it is a plain buffered merge. Once the blocks of a merge outgrow it the
/// local merges rotate, so the worst case is the rotation merge's.
pub struct FixedBuffer;
impl FixedBuffer {
    pub const CAPACITY: usize = 512;
}
impl Auxiliary for FixedBuffer {
    const NAME: &'static str = "fixed";
    const MERGE_WORST: Complexity = Complexity::N_LOG_N;
    const MERGE_SPACE: Complexity = Complexity::CONST;

    fn scratch_size(n: usize) -> usize { n.min(Self::CAPACITY) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_in_array_buffer_allocates_nothing_and_carves_a_square_root() {
        assert_eq!(InArrayBuffer::scratch_size(10_000), 0);
        assert_eq!(InArrayBuffer::internal_size(10_000), 100);
        assert_eq!(InArrayBuffer::internal_size(99), 9);
        assert_eq!(InArrayBuffer::internal_size(0), 0);
    }

    #[test]
    fn the_fixed_buffer_stops_at_its_capacity_and_carves_nothing() {
        assert_eq!(FixedBuffer::scratch_size(100), 100);
        assert_eq!(FixedBuffer::scratch_size(512), 512);
        assert_eq!(FixedBuffer::scratch_size(100_000), FixedBuffer::CAPACITY);
        assert_eq!(FixedBuffer::internal_size(100_000), 0);
    }

    #[test]
    fn the_existing_backings_carve_nothing() {
        assert_eq!(crate::FullBuffer::internal_size(100), 0);
        assert_eq!(crate::HalfBuffer::internal_size(100), 0);
        assert_eq!(crate::NoBuffer::internal_size(100), 0);
    }
}
