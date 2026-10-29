//! The two `Auxiliary` backings of a block merge sort, each sized by a
//! `BufferSize`. `InArrayBuffer` carves its buffer out of the array, so it
//! allocates nothing and its values are swapped with real elements.
//! `ExternalBuffer` is an aux array, a free overwrite. Either holds a block of
//! every merge when its size keeps up with the square root of the length, and
//! the local merges of the blocks that outgrow a smaller one rotate.

use core::marker::PhantomData;

use array_vis_bench_traits::role::{Auxiliary, BufferSize};
use array_vis_bench_traits::Complexity;

// A merge costs `O(n)` with a buffer that holds its blocks, and the rotation
// merge's `O(n log n)` with one that does not.
const fn merge_worst(size: Complexity) -> Complexity {
    if size.at_least(Complexity::SQRT_N) { Complexity::N1 } else { Complexity::N_LOG_N }
}

/// `S::cells(n)` distinct elements of the array serve as the merge buffer, `O(1)`
/// space. Writing to it is a swap, and the buffer is sorted back into the array
/// when the sort ends.
pub struct InArrayBuffer<S>(PhantomData<S>);
impl<S: BufferSize> Auxiliary for InArrayBuffer<S> {
    const NAME: &'static str = "in array";
    const MERGE_WORST: Complexity = merge_worst(S::GROWTH);
    const MERGE_SPACE: Complexity = Complexity::CONST;
    const CARVED: Complexity = S::GROWTH;

    fn scratch_size(_n: usize) -> usize { 0 }

    fn internal_size(n: usize) -> usize { S::cells(n) }
}

/// An aux array of `S::cells(n)` cells, never more than the array.
pub struct ExternalBuffer<S>(PhantomData<S>);
impl<S: BufferSize> Auxiliary for ExternalBuffer<S> {
    const NAME: &'static str = "external";
    const MERGE_WORST: Complexity = merge_worst(S::GROWTH);
    const MERGE_SPACE: Complexity = S::GROWTH;

    fn scratch_size(n: usize) -> usize { S::cells(n).min(n) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Fixed, Half, Sqrt};

    #[test]
    fn the_in_array_buffer_allocates_nothing_and_carves_its_size() {
        assert_eq!(InArrayBuffer::<Sqrt>::scratch_size(10_000), 0);
        assert_eq!(InArrayBuffer::<Sqrt>::internal_size(10_000), 100);
        assert_eq!(InArrayBuffer::<Half>::internal_size(99), 50);
        assert_eq!(InArrayBuffer::<Fixed<512>>::internal_size(100_000), 512);
    }

    #[test]
    fn the_external_buffer_stops_at_the_array_and_carves_nothing() {
        assert_eq!(ExternalBuffer::<Fixed<512>>::scratch_size(100), 100);
        assert_eq!(ExternalBuffer::<Fixed<512>>::scratch_size(100_000), 512);
        assert_eq!(ExternalBuffer::<Sqrt>::scratch_size(10_000), 100);
        assert_eq!(ExternalBuffer::<Half>::internal_size(100_000), 0);
    }

    #[test]
    fn a_buffer_that_keeps_up_with_the_square_root_merges_in_linear_time() {
        assert_eq!(InArrayBuffer::<Sqrt>::MERGE_WORST, Complexity::N1);
        assert_eq!(ExternalBuffer::<Half>::MERGE_WORST, Complexity::N1);
        assert_eq!(InArrayBuffer::<Fixed<512>>::MERGE_WORST, Complexity::N_LOG_N);
        assert_eq!(ExternalBuffer::<Fixed<512>>::MERGE_WORST, Complexity::N_LOG_N);
    }

    #[test]
    fn only_the_external_buffer_takes_space_and_only_the_carved_one_costs_carving() {
        assert_eq!(InArrayBuffer::<Half>::MERGE_SPACE, Complexity::CONST);
        assert_eq!(ExternalBuffer::<Half>::MERGE_SPACE, Complexity::N1);
        assert_eq!(ExternalBuffer::<Sqrt>::MERGE_SPACE, Complexity::SQRT_N);
        assert_eq!(InArrayBuffer::<Half>::CARVED, Complexity::N1);
        assert_eq!(ExternalBuffer::<Half>::CARVED, Complexity::CONST);
    }

    #[test]
    fn the_existing_backings_carve_nothing() {
        assert_eq!(crate::FullBuffer::internal_size(100), 0);
        assert_eq!(crate::HalfBuffer::internal_size(100), 0);
        assert_eq!(crate::NoBuffer::internal_size(100), 0);
    }
}
