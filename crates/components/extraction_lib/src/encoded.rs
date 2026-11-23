//! `EncodedPartition`, the stable partition in linear time: the region is
//! gathered into blocks of one type each, and the blocks are put in order by
//! the `blocks` module, which writes which block was which into the elements
//! themselves. `gather` makes the blocks, with a buffer of a block for the back
//! elements, and `by_encoding` is the whole partition.

use array_vis_bench_traits::role::{Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::blocks::{ceil_log2, finish, trim};
use crate::partition::buffered;

pub struct EncodedPartition;
impl StablePartition for EncodedPartition {
    const NAME: &'static str = "encoded";
    const MOVES: Complexity = Complexity::N1;

    fn scratch_size(n: usize) -> usize { block_len(n) }

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        _words: &mut [usize],
        log: &mut U,
    ) -> usize {
        by_encoding::<R, T, U, _>(arr, lo, hi, block_len(hi - lo), &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), scratch, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        _words: &mut [usize],
        log: &mut U,
    ) -> usize {
        by_encoding::<R, T, U, _>(arr, lo, hi, block_len(hi - lo), &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), scratch, log)
    }
}

// Cells in a block, and in the buffer that holds the back elements of the one
// being gathered: BLOCK_LOGS times the logarithm of `n`, rounded up, and at least
// two. The minority type has at most half of the at most `n / 2` blocks, so its
// ranks take `ceil(log2(n))` bits or fewer, and the last cell of a block, which
// tells its type, is never one of the cells that carry a rank. One time the
// logarithm is enough for that and eight times is faster: a block that long cuts
// the blocks to an eighth, and with them the type reads, the rank reads and the
// scattered block swaps. Measured on 2^20 random elements half of them in front,
// one partition took 10.8 ms at one, 7.1 at two, 6.0 at four, 5.2 at eight and
// 5.2 at sixteen times the logarithm, with 1.9, 1.4, 1.2, 1.1 and 1.1
// comparisons per element, and the moves moved from 3.6 to 2.8 per element.
const BLOCK_LOGS: usize = 8;

fn block_len(n: usize) -> usize { if n < 2 { 0 } else { (BLOCK_LOGS * ceil_log2(n)).max(2) } }

// `front` says whether the element at an index is one of those to bring to the
// front, by one logged comparison against the pivot, and `b` is the block length.
// The ends in place are left by `trim`, the blocks are made by `gather`, and put
// in order, with the loose elements placed, by `finish`.
fn by_encoding<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    b: usize,
    front: &F,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    let (lo, hi) = match trim(arr, lo, hi, front, log) {
        Ok(region) => region,
        Err(split) => return split,
    };
    if hi - lo <= scratch.len() { return buffered(arr, lo, hi, front, scratch, log); }
    assert!(scratch.len() >= b, "a partition of {} elements needs {b} cells of scratch, not {}", hi - lo, scratch.len());
    let (end, loose, held) = gather(arr, lo, hi, b, front, scratch, log);
    // The back elements held go to the free cells behind the loose front ones,
    // where they belong.
    log.copy_range(scratch, 0, arr, end + loose, held);
    finish::<R, T, U, F>(arr, lo, end, loose, b, front, scratch, log)
}

// One scan from the left leaves whole blocks of one type in `arr[lo..end]`,
// each type's in order, then `loose` front elements, fewer than a block, and
// the `held` back elements, fewer than a block, in the scratch. A front element
// is copied down to follow the loose ones, a back element is held, and the
// cells between the loose elements and the scan are free, as many as are held:
// when a block of the back elements is whole, the loose ones step up over it
// into the free cells and the block is written where they were. Two moves for
// each element at most, and one comparison.
//
// It stays out of line. Inlined into `by_encoding`, with the trim and the finish
// shared with the masked partitions, its loop ran 40 to 90 percent slower: one
// partition of 2^20 random elements took 7 to 10 ms against 5 out of line.
#[inline(never)]
fn gather<T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    b: usize,
    front: &F,
    scratch: &mut [T],
    log: &mut U,
) -> (usize, usize, usize) {
    let (mut end, mut loose, mut held) = (lo, 0, 0);
    for i in lo..hi {
        if front(log, arr, i) {
            if end + loose != i { log.write(arr, end + loose, i); }
            loose += 1;
            if loose == b { (end, loose) = (end + b, 0); }
        } else {
            log.write_accross(arr, i, scratch, held);
            held += 1;
            if held == b {
                for k in 0..loose { log.write(arr, end + b + k, end + k); }
                log.copy_range(scratch, 0, arr, end, b);
                (end, held) = (end + b, 0);
            }
        }
    }
    (end, loose, held)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::*;
    use rotation_auxiliary::AuxiliaryRotation;
    use rotation_block_cycle::BlockCycleRotation;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
    use sort_logger::SortLogger;

    // The partition with blocks of the least length that works, the logarithm of
    // the region, which has the many small blocks of a short region and the tight
    // arithmetic of the code cells that the production length has not.
    struct Tight;
    impl StablePartition for Tight {
        const NAME: &'static str = "tight";
        const MOVES: Complexity = Complexity::N1;

        fn scratch_size(n: usize) -> usize { if n < 2 { 0 } else { ceil_log2(n).max(2) } }

        fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            pivot: T,
            scratch: &mut [T],
            _words: &mut [usize],
            log: &mut U,
        ) -> usize {
            let b = Self::scratch_size(hi - lo);
            by_encoding::<R, T, U, _>(arr, lo, hi, b, &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), scratch, log)
        }

        fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            pivot: T,
            scratch: &mut [T],
            _words: &mut [usize],
            log: &mut U,
        ) -> usize {
            let b = Self::scratch_size(hi - lo);
            by_encoding::<R, T, U, _>(arr, lo, hi, b, &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), scratch, log)
        }
    }

    // The block length the shapes of input are laid out in.
    fn tight(n: usize) -> usize { Tight::scratch_size(n) }

    #[test]
    fn the_encoded_partition_matches_the_reference() {
        matches_the_reference::<EncodedPartition, ReversalRotation>(0, tight);
        matches_the_reference::<EncodedPartition, TrinityRotation>(0, tight);
        matches_the_reference::<EncodedPartition, AuxiliaryRotation>(0, tight);
        matches_the_reference::<EncodedPartition, BlockCycleRotation>(0, tight);
    }

    #[test]
    fn blocks_of_the_least_length_match_the_reference() {
        matches_the_reference::<Tight, ReversalRotation>(0, tight);
        matches_the_reference::<Tight, TrinityRotation>(0, tight);
        matches_the_reference::<Tight, AuxiliaryRotation>(0, tight);
        matches_the_reference::<Tight, BlockCycleRotation>(0, tight);
    }

    // A scratch longer than the one asked for, up to one that holds the whole
    // region, which the buffered pass takes.
    #[test]
    fn a_longer_scratch_matches_the_reference() {
        for extra in [5, 40, 400] {
            matches_the_reference::<EncodedPartition, ReversalRotation>(extra, tight);
            matches_the_reference::<Tight, ReversalRotation>(extra, tight);
        }
    }

    #[test]
    fn the_code_cells_leave_the_type_cell_alone() {
        code_cells_fit(block_len);
        code_cells_fit(Tight::scratch_size);
    }

    #[test]
    fn no_move_goes_unlogged_and_no_cell_is_read_before_it_is_written() {
        replays_without_a_gap::<EncodedPartition>(tight);
    }

    #[test]
    fn the_moves_per_element_are_flat_in_the_size_and_below_the_rotation_partitions() {
        moves_are_flat_and_below_the_rotation_partition::<EncodedPartition>(tight, 1 << 10);
    }

    #[test]
    fn every_shape_moves_a_few_times_per_element() { every_shape_is_cheap::<EncodedPartition>(tight); }

    #[test]
    fn a_pass_with_nothing_to_move_costs_comparisons_only() {
        nothing_to_move_costs_comparisons_only::<EncodedPartition>(tight);
    }

    #[test]
    fn the_ends_that_are_in_place_cost_no_moves() { ends_in_place_cost_no_moves::<EncodedPartition>(); }

    #[test]
    fn an_empty_region_and_a_lone_element_cost_nothing() { empty_and_lone_regions_cost_nothing::<EncodedPartition>(); }
}
