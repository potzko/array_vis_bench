//! `EncodedPartition`, the stable partition in linear time: the region is
//! gathered into blocks of one type each, the blocks are put in order by block
//! swaps, and the one thing a block swap cannot do, remember which block was
//! which, is written into the elements themselves, one bit at a time, by
//! swapping two elements the pivot tells apart. `gather` makes the blocks,
//! `order_blocks` orders them and `by_encoding` is the whole partition, which
//! first leaves the front elements at the start and the back ones at the end
//! where they are.

use array_vis_bench_traits::role::keys::rotate_region;
use array_vis_bench_traits::role::{Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

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

fn ceil_log2(n: usize) -> usize { (usize::BITS - (n - 1).leading_zeros()) as usize }

// `front` says whether the element at an index is one of those to bring to the
// front, by one logged comparison against the pivot, and `b` is the block length.
// The front elements the region starts with and the back elements it ends with
// are in place, and are read once and left, so a pass with nothing to move costs
// its comparisons alone; what is between them, which starts with a back element
// and ends with a front one, is partitioned.
fn by_encoding<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    b: usize,
    front: &F,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    let Some(start) = (lo..hi).find(|&i| !front(log, arr, i)) else { return hi };
    let Some(last) = (start + 1..hi).rev().find(|&i| front(log, arr, i)) else { return start };
    let (lo, hi) = (start, last + 1);
    let n = hi - lo;
    if n <= scratch.len() { return buffered(arr, lo, hi, front, scratch, log); }
    assert!(scratch.len() >= b, "a partition of {n} elements needs {b} cells of scratch, not {}", scratch.len());
    let (end, loose, held) = gather(arr, lo, hi, b, front, scratch, log);
    let fronts = order_blocks(arr, lo, (end - lo) / b, b, front, log);
    // The back elements held go to the free cells behind the loose front ones,
    // where they belong, and the loose ones rotate in front of the back blocks.
    log.copy_range(scratch, 0, arr, end + loose, held);
    rotate_region::<R, T, U>(arr, lo + fronts * b, end, end + loose, scratch, log);
    lo + fronts * b + loose
}

// One scan from the left leaves whole blocks of one type in `arr[lo..end]`,
// each type's in order, then `loose` front elements, fewer than a block, and
// the `held` back elements, fewer than a block, in the scratch. A front element
// is copied down to follow the loose ones, a back element is held, and the
// cells between the loose elements and the scan are free, as many as are held:
// when a block of the back elements is whole, the loose ones step up over it
// into the free cells and the block is written where they were. Two moves for
// each element at most, and one comparison.
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

// Put the `m` blocks of `b` cells from `lo` in order, the front blocks first,
// each type's in the order it had, and return how many are front. A block's
// type is the type of its last cell, which nothing below touches.
//
// What a block swap cannot do is remember which block was which, and the
// blocks of the minority type are the ones that get scrambled, so they carry
// their rank with them. A minority block and a majority block that are the
// `j`th of their type make a pair, and the bits of `j` are written into the
// first cells of the two by swapping a cell of one with the same cell of the
// other for each bit that is set: the pivot tells the two elements apart, so
// the minority block then holds a majority element in exactly the cells of the
// bits of its rank, and no element has to be distinct from another. The
// majority blocks sweep to their end of the row by block swaps, in order, past
// a gap of minority blocks that comes out scrambled, the minority blocks read
// their ranks off the cells and swap into place by cycle leader, each swap
// settling one block, and the swaps of the code are undone, which needs no
// comparison since the block at `j` has the rank `j`.
fn order_blocks<T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    m: usize,
    b: usize,
    front: &F,
    log: &mut U,
) -> usize {
    let cell = |block: usize, t: usize| lo + block * b + t;
    let is_front = |log: &mut U, arr: &[T], block: usize| front(log, arr, cell(block, b - 1));
    let fronts = (0..m).filter(|&block| is_front(log, arr, block)).count();
    let minors = fronts.min(m - fronts);
    if minors == 0 { return fronts; }
    let minor_front = fronts < m - fronts;
    let bits = ceil_log2(minors);
    debug_assert!(bits < b, "{minors} minority blocks need {bits} code cells in blocks of {b}");
    let set = |j: usize| (0..bits).filter(move |t| (j >> t) & 1 == 1);
    // Pair up and write the ranks.
    if bits > 0 {
        let next = |log: &mut U, arr: &[T], from: usize, wanted: bool| (from..m).find(|&block| is_front(log, arr, block) == wanted).unwrap();
        let (mut f, mut k) = (0, 0);
        for j in 0..minors {
            (f, k) = (next(log, arr, f, true), next(log, arr, k, false));
            set(j).for_each(|t| log.swap(arr, cell(f, t), cell(k, t)));
            (f, k) = (f + 1, k + 1);
        }
    }
    // Sweep the majority blocks to one end, from the left when they are front
    // blocks and from the right when they are back blocks.
    let at = |s: usize| if minor_front { m - 1 - s } else { s };
    let mut placed = 0;
    for s in 0..m {
        if is_front(log, arr, at(s)) != minor_front {
            if placed != s { log.block_swap(arr, cell(at(placed), 0), cell(at(s), 0), b); }
            placed += 1;
        }
    }
    // The minority blocks are at the other end, in no order.
    let base = if minor_front { 0 } else { fronts };
    for p in base..base + minors {
        loop {
            let rank = (0..bits).filter(|&t| front(log, arr, cell(p, t)) != minor_front).fold(0, |rank, t| rank | 1 << t);
            if rank == p - base { break; }
            log.block_swap(arr, cell(p, 0), cell(base + rank, 0), b);
        }
    }
    for j in 0..minors { set(j).for_each(|t| log.swap(arr, cell(j, t), cell(fronts + j, t))); }
    fronts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{cost, item, items, matches_the_reference, types, Replay, Rng, Tally, SHAPES};
    use crate::RotationPartition;
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

    // The block length leaves the cell that tells a block's type alone: the
    // code cells are `ceil(log2(minority blocks))` of the cells, the minority is
    // at most half of the blocks, and a block is longer than that.
    #[test]
    fn the_code_cells_leave_the_type_cell_alone() {
        for n in (2..20_000).chain([1 << 20, (1 << 20) + 1, 1 << 30, usize::MAX >> 1]) {
            for b in [block_len(n), Tight::scratch_size(n)] {
                let minority = n / b / 2;
                assert!(b >= 2 && (minority < 2 || ceil_log2(minority) < b), "n={n}: {b} cells for {minority} minority blocks");
            }
        }
    }

    // Moves and comparisons that count are the logged ones, and a cell of the
    // scratch is written before it is read: the scratch is full of garbage the
    // replay's copy of it lacks, so a read of an unwritten cell would leave the
    // two arrays apart.
    #[test]
    fn no_move_goes_unlogged_and_no_cell_is_read_before_it_is_written() {
        const BACK: usize = 1 << 20;
        let mut rng = Rng(0x10661);
        for n in [3usize, 10, 33, 100, 257, 1000, 3001] {
            for shape in 0..SHAPES {
                let mut arr: Vec<usize> = types(shape, n, tight(n), &mut rng).iter().zip(0..).map(|(&f, i)| if f { i } else { BACK + i }).collect();
                let mut replay = Replay::of(&arr);
                let mut scratch = vec![usize::MAX; EncodedPartition::scratch_size(n)];
                replay.log_aux_arr_t(&scratch);
                let split = EncodedPartition::partition::<ReversalRotation, _, _>(&mut arr, 0, n, BACK, &mut scratch, &mut replay);
                assert!(arr[..split].iter().all(|&x| x < BACK) && arr[split..].iter().all(|&x| x >= BACK), "n={n} shape={shape}");
                assert!(arr[..split].windows(2).all(|w| w[0] < w[1]) && arr[split..].windows(2).all(|w| w[0] < w[1]), "n={n} shape={shape}: order");
                assert_eq!(replay.main(), &arr[..], "n={n} shape={shape}: a move went unlogged or an unwritten cell was read");
            }
        }
    }

    // The moves per element do not grow with the size, which a partition by
    // rotation's `log n` would, and stay under a few for every shape.
    #[test]
    fn the_moves_per_element_are_flat_in_the_size_and_below_the_rotation_partitions() {
        let (small, _) = cost::<EncodedPartition, ReversalRotation>(1 << 10, 2, tight(1 << 10));
        let (large, cmps) = cost::<EncodedPartition, ReversalRotation>(1 << 16, 2, tight(1 << 16));
        let (rotation, _) = cost::<RotationPartition, ReversalRotation>(1 << 16, 2, tight(1 << 16));
        assert!(large < 1.3 * small, "{small} moves per element at 2^10 and {large} at 2^16");
        assert!(large < 5.0, "{large} moves per element at 2^16");
        assert!(cmps < 1.3, "{cmps} comparisons per element at 2^16");
        assert!(2.0 * large < rotation, "{large} moves per element against the rotation partition's {rotation}");
    }

    #[test]
    fn every_shape_moves_a_few_times_per_element() {
        for shape in 0..SHAPES {
            let (moves, cmps) = cost::<EncodedPartition, ReversalRotation>(1 << 14, shape, tight(1 << 14));
            assert!(moves < 5.0 && cmps < 1.5, "shape {shape}: {moves} moves and {cmps} comparisons per element");
        }
    }

    // A region already partitioned, or with nothing to move to the front, costs
    // its comparisons and no move: every element is read once and none written.
    #[test]
    fn a_pass_with_nothing_to_move_costs_comparisons_only() {
        for n in [100usize, 1 << 10, 1 << 14, 40_000] {
            for (shape, name) in [(0, "all front"), (1, "all back"), (7, "sorted")] {
                let mut rng = Rng(5);
                let types = types(shape, n, tight(n), &mut rng);
                let mut arr: Vec<usize> = types.iter().enumerate().map(|(i, &f)| if f { i } else { n + i }).collect();
                let want = arr.clone();
                let mut scratch = vec![0usize; EncodedPartition::scratch_size(n)];
                let mut tally = Tally::default();
                let split = EncodedPartition::partition::<ReversalRotation, _, _>(&mut arr, 0, n, n, &mut scratch, &mut tally);
                assert_eq!(arr, want, "{name}, n={n}: moved");
                assert_eq!(split, types.iter().filter(|&&f| f).count(), "{name}, n={n}: the split");
                assert_eq!((tally.moves, tally.cmps <= n + 2), (0, true), "{name}, n={n}: {} moves, {} comparisons", tally.moves, tally.cmps);
            }
        }
    }

    // The ends are trimmed, and what is between them partitioned: an eighth in
    // front, a quarter shuffled and the rest in back cost the quarter alone.
    #[test]
    fn the_ends_that_are_in_place_cost_no_moves() {
        let n = 1 << 14;
        let mut rng = Rng(11);
        let middle: Vec<usize> = (0..n / 4).map(|i| if rng.next(2) == 0 { i } else { n + i }).collect();
        let mut arr: Vec<usize> = (0..n / 8).chain(middle).chain((0..5 * n / 8).map(|i| n + i)).collect();
        let mut scratch = vec![0usize; EncodedPartition::scratch_size(n)];
        let mut tally = Tally::default();
        let split = EncodedPartition::partition::<ReversalRotation, _, _>(&mut arr, 0, n, n, &mut scratch, &mut tally);
        assert!(arr[..split].iter().all(|&x| x < n) && arr[split..].iter().all(|&x| x >= n));
        assert!(tally.moves < n, "{} moves for a middle of {}", tally.moves, n / 4);
        assert!(tally.cmps < n + n / 16, "{} comparisons", tally.cmps);
    }

    #[test]
    fn an_empty_region_and_a_lone_element_cost_nothing() {
        let mut arr = items(&[3, 1]);
        let mut tally = Tally::default();
        let empty = EncodedPartition::partition::<ReversalRotation, _, _>(&mut arr, 1, 1, item(2), &mut [], &mut tally);
        assert_eq!((empty, tally.moves, tally.cmps), (1, 0, 0));
        let lone = EncodedPartition::partition::<ReversalRotation, _, _>(&mut arr, 1, 2, item(2), &mut [], &mut tally);
        assert_eq!((lone, tally.moves, tally.cmps), (2, 0, 1));
    }
}
