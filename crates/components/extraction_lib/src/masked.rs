//! `MaskedPartition`, the stable partition in linear time with no buffer: the
//! whole blocks that `EncodedPartition` gathers through a buffer are made here
//! by permuting windows of the region, each window's classes held as a bit mask
//! in a few local words. `permute_window` reads a window's mask and works out
//! where every element goes from popcounts of the mask below it, into a table of
//! places, `cycle_permute` applies the table by cycles with one element carried
//! in a local, `windows` slides the window along the region, and `by_masks` is
//! the whole partition, the `blocks` module ordering the blocks the windows
//! leave.

use array_vis_bench_traits::role::{Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::blocks::{by_blocks, ceil_log2};

/// Windows of the region permuted by bit masks into whole blocks, then the
/// blocks put in order as `EncodedPartition` does: `O(n)` moves and
/// comparisons, no scratch of its own.
pub struct MaskedPartition;
impl StablePartition for MaskedPartition {
    const NAME: &'static str = "masked";
    const MOVES: Complexity = Complexity::N1;

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        let b = block_len(hi - lo);
        by_masks::<R, T, U, _>(arr, lo, hi, b, window(b), &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), scratch, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        log: &mut U,
    ) -> usize {
        let b = block_len(hi - lo);
        by_masks::<R, T, U, _>(arr, lo, hi, b, window(b), &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), scratch, log)
    }
}

// Words of a mask, and so the most elements a window can hold, 1024: the two
// partial blocks a window starts with and the fresh elements it takes. A window
// is permuted out of two masks, the classes and the positions done, and a table
// of how many backs lie below each word, which are the only memory it uses.
const MASK_WORDS: usize = 16;

// Cells in a block: BLOCK_LOGS times the logarithm of `n`, rounded up, and at
// least two, as in `EncodedPartition`, and at most a quarter of a window's
// bits, so that a window holds at least two blocks' worth of fresh elements and
// every block count of a region fits the code cells of its ranks.
const BLOCK_LOGS: usize = 8;
const MAX_BLOCK: usize = MASK_WORDS * 64 / 4;

fn block_len(n: usize) -> usize { if n < 2 { 0 } else { (BLOCK_LOGS * ceil_log2(n)).clamp(2, MAX_BLOCK) } }

// Fresh elements a window takes on top of its partial blocks of fewer than `b`
// cells each, so that the whole window fits the masks.
fn window(b: usize) -> usize { MASK_WORDS * 64 - 2 * b }

// `front` says whether the element at an index is one of those to bring to the
// front, by one logged comparison against the pivot, `b` is the block length and
// `w` the fresh elements a window takes. The trim and everything after the blocks
// are made belong to `by_blocks`.
fn by_masks<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    b: usize,
    w: usize,
    front: &F,
    scratch: &mut [T],
    log: &mut U,
) -> usize {
    by_blocks::<R, T, U, F, _>(arr, lo, hi, b, front, scratch, log, |arr, lo, hi, _, log| windows(arr, lo, hi, b, w, front, log))
}

// The region goes through in windows. Behind the window are whole blocks of one
// type each, `arr[lo..end]`, and then the partial blocks, `z` front elements
// followed by `h` back elements, which start the next window, with the next
// `w` elements still unread behind them. Each window is permuted into whole
// blocks and new partial blocks, so each element is compared once and moved
// about once, and nothing outside the masks is kept. Returns `end` and `z`, and
// the `h` back elements run to `hi`, as `by_blocks` asks.
fn windows<T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    b: usize,
    w: usize,
    front: &F,
    log: &mut U,
) -> (usize, usize) {
    let (mut end, mut z, mut h) = (lo, 0, 0);
    while end + z + h < hi {
        let fresh = w.min(hi - end - z - h);
        let (whole, fronts, backs) = permute_window(arr, end, (z, h, fresh), b, front, log);
        (end, z, h) = (end + whole, fronts, backs);
    }
    (end, z)
}

// The window is `arr[start..]` for `z + h + fresh` elements: `z` front elements,
// `h` back elements and `fresh` unread ones, which are read here, once each. It
// comes out as whole blocks of the window's front elements, whole blocks of its
// back elements, then the front elements and then the back elements that are
// left, fewer than a block each; a front element of the window's rank `r` among
// the fronts goes to `r` while `r` is below the front blocks' cells and after the
// back blocks otherwise, and a back element likewise, so the blocks take each
// type's earliest elements and the loose ones the latest, each type in order, and
// the partial blocks the window started with are the earliest of their types, so
// they land in the whole blocks first. A rank is a popcount of the mask below the
// position, so the mask does not change while the elements move.
//
// The places are worked out for every position before any element moves, into a
// table of 16 bit entries, one a position, which holds places and never
// elements. A cycle walk that worked each place out as it went waited for a
// load, a popcount and the selects of every step before it could start the
// next, and took 6.9 ms over 2^20 random elements; walking a table, which is
// filled in a pass where no entry waits for another, took 3.0. Returns the cells
// in whole blocks, and the front and back elements left over.
fn permute_window<T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    start: usize,
    (z, h, fresh): (usize, usize, usize),
    b: usize,
    front: &F,
    log: &mut U,
) -> (usize, usize, usize) {
    let len = z + h + fresh;
    assert!(len <= MASK_WORDS * 64, "a window of {len} elements does not fit {MASK_WORDS} words");
    // Bit `p` of the mask is set when the element at `p` is a back element. A
    // word is built in a register and stored once: a store to the same word for
    // each element is a chain through memory, and it cost four times as much.
    let words = len.div_ceil(64);
    let mut back = [0u64; MASK_WORDS];
    for (word, bits) in back[..words].iter_mut().enumerate() {
        let (from, to) = (word * 64, (word * 64 + 64).min(len));
        let held = (from.max(z)..to.min(z + h)).fold(0u64, |bits, p| bits | 1 << (p % 64));
        *bits = (from.max(z + h)..to).fold(held, |bits, p| bits | u64::from(!front(log, arr, start + p)) << (p % 64));
    }
    // `below[i]` is the back elements in the words before word `i`.
    let mut below = [0usize; MASK_WORDS + 1];
    (0..words).for_each(|i| below[i + 1] = below[i] + back[i].count_ones() as usize);
    let (backs, fronts) = (below[words], len - below[words]);
    let (whole_fronts, whole_backs) = (fronts / b * b, backs / b * b);
    let (blocks, loose_fronts) = (whole_fronts + whole_backs, fronts - whole_fronts);
    let place = |p: usize| {
        let backs_before = below[p / 64] + (back[p / 64] & ((1u64 << (p % 64)) - 1)).count_ones() as usize;
        let (rank, whole, first_loose, loose_base) = if back[p / 64] >> (p % 64) & 1 == 1 {
            (backs_before, whole_backs, whole_fronts, blocks + loose_fronts)
        } else {
            (p - backs_before, whole_fronts, 0, blocks)
        };
        if rank < whole { first_loose + rank } else { loose_base + rank - whole }
    };
    let mut dest = [0u16; MASK_WORDS * 64];
    dest[..len].iter_mut().enumerate().for_each(|(p, to)| *to = place(p) as u16);
    cycle_permute(arr, start, &mut dest[..len], log);
    (blocks, loose_fronts, backs - whole_backs)
}

// Moves the element at each position `p` of `arr[start..]` to `dest[p]`, which
// is a permutation of the positions. A cycle is followed with its element
// carried in a local: take the first position not done whose place is elsewhere,
// write what it holds to its place, carry the element that was there and write
// it to its place in turn, until the cycle closes on the first position. Each
// element that moves is written once, by `write_data`, and a position done is
// marked by pointing its entry at itself.
fn cycle_permute<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], start: usize, dest: &mut [u16], log: &mut U) {
    for first in 0..dest.len() {
        if dest[first] as usize == first { continue; }
        let (mut at, mut carried) = (first, arr[start + first]);
        loop {
            let to = dest[at] as usize;
            dest[at] = at as u16;
            let displaced = arr[start + to];
            log.write_data(arr, start + to, carried);
            if to == first { break; }
            (at, carried) = (to, displaced);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::*;
    use crate::EncodedPartition;
    use rotation_auxiliary::AuxiliaryRotation;
    use rotation_block_cycle::BlockCycleRotation;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
    use sort_logger::SortLogger;

    // The block length of the least length that works, the logarithm of the
    // region, which makes the many small blocks of a short region.
    fn tight(n: usize) -> usize { if n < 2 { 0 } else { ceil_log2(n).max(2) } }

    // The partition with blocks of `B` cells, or of the least length when `B` is
    // 0, and windows of `W` fresh elements, so that a window can end at every
    // place a block can.
    struct Windowed<const B: usize, const W: usize>;
    impl<const B: usize, const W: usize> Windowed<B, W> {
        fn block(n: usize) -> usize { if B == 0 { tight(n) } else { B } }
    }
    impl<const B: usize, const W: usize> StablePartition for Windowed<B, W> {
        const NAME: &'static str = "masked, sized";
        const MOVES: Complexity = Complexity::N1;

        fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            pivot: T,
            scratch: &mut [T],
            log: &mut U,
        ) -> usize {
            by_masks::<R, T, U, _>(arr, lo, hi, Self::block(hi - lo), W, &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), scratch, log)
        }

        fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            pivot: T,
            scratch: &mut [T],
            log: &mut U,
        ) -> usize {
            by_masks::<R, T, U, _>(arr, lo, hi, Self::block(hi - lo), W, &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), scratch, log)
        }
    }

    const BACK: usize = 1 << 20;

    // A window of `z` fronts, `h` backs and some unread elements comes out as
    // the whole blocks of fronts, the whole blocks of backs, the latest fronts
    // and the latest backs, each in order, with every unread element compared
    // once, every element that moves written once and none that stays touched.
    #[test]
    fn a_window_comes_out_as_whole_blocks_then_the_loose_elements() {
        let mut rng = Rng(0x57A6);
        let is_front = |log: &mut Tally, arr: &[usize], i: usize| log.cmp_lt_data(arr, i, BACK);
        for b in [2usize, 3, 5, 16, 100] {
            for round in 0..3000 {
                let (z, h) = (rng.next(b), rng.next(b));
                let fresh = 1 + rng.next([3, 70, 1024][round % 3].min(MASK_WORDS * 64 - z - h));
                let odds = 1 + rng.next(9);
                let kinds: Vec<bool> = (0..z).map(|_| true).chain((0..h).map(|_| false)).chain((0..fresh).map(|_| rng.next(odds) != 0)).collect();
                let before: Vec<usize> = kinds.iter().zip(0..).map(|(&f, i)| if f { i } else { BACK + i }).collect();
                let start = rng.next(5);
                let mut arr: Vec<usize> = vec![7; start].into_iter().chain(before.iter().copied()).chain([8, 9]).collect();
                let fronts: Vec<usize> = before.iter().copied().filter(|&x| x < BACK).collect();
                let backs: Vec<usize> = before.iter().copied().filter(|&x| x >= BACK).collect();
                let (whole_fronts, whole_backs) = (fronts.len() / b * b, backs.len() / b * b);
                let expected: Vec<usize> = fronts[..whole_fronts].iter().chain(&backs[..whole_backs]).chain(&fronts[whole_fronts..]).chain(&backs[whole_backs..]).copied().collect();
                let mut tally = Tally::default();
                let got = permute_window(&mut arr, start, (z, h, fresh), b, &is_front, &mut tally);
                let name = format!("b={b} z={z} h={h} fresh={fresh}");
                assert_eq!(&arr[start..start + before.len()], &expected[..], "{name}: the window");
                assert_eq!((&arr[..start], &arr[start + before.len()..]), (&vec![7; start][..], &[8, 9][..]), "{name}: outside the window");
                assert_eq!(got, (whole_fronts + whole_backs, fronts.len() - whole_fronts, backs.len() - whole_backs), "{name}: what is left over");
                let moved = before.iter().zip(&expected).filter(|(a, b)| a != b).count();
                assert_eq!((tally.moves, tally.cmps), (moved, fresh), "{name}: moves and comparisons");
            }
        }
    }

    // A permutation by cycles puts every element where `place` says, writing
    // each element that moves once and logging every write.
    #[test]
    fn a_permutation_by_cycles_writes_each_moved_element_once() {
        let mut rng = Rng(0xC1C1E);
        for len in [0usize, 1, 2, 3, 63, 64, 65, 500, MASK_WORDS * 64] {
            for round in 0..20 {
                let mut place: Vec<usize> = (0..len).collect();
                match round % 4 {
                    0 => (0..3.min(len)).for_each(|_| place.swap(rng.next(len), rng.next(len))),
                    _ => (1..len).rev().for_each(|i| place.swap(i, rng.next(i + 1))),
                }
                let start = rng.next(4);
                let before: Vec<usize> = (0..start + len + 2).collect();
                let mut arr = before.clone();
                let mut replay = Replay::of(&arr);
                cycle_permute(&mut arr, start, &mut place.iter().map(|&p| p as u16).collect::<Vec<_>>(), &mut replay);
                assert!((0..len).all(|p| arr[start + place[p]] == before[start + p]), "len={len} round={round}: not placed");
                assert_eq!((&arr[..start], &arr[start + len..]), (&before[..start], &before[start + len..]), "len={len}: outside");
                assert_eq!(replay.main(), &arr[..], "len={len} round={round}: a move went unlogged");
                let mut tally = Tally::default();
                let mut again = before.clone();
                cycle_permute(&mut again, start, &mut place.iter().map(|&p| p as u16).collect::<Vec<_>>(), &mut tally);
                assert_eq!(tally.moves, (0..len).filter(|&p| place[p] != p).count(), "len={len} round={round}: moves");
            }
        }
    }

    #[test]
    fn the_masked_partition_matches_the_reference() {
        for shapes in [block_len, tight] {
            matches_the_reference::<MaskedPartition, ReversalRotation>(0, shapes);
            matches_the_reference::<MaskedPartition, TrinityRotation>(0, shapes);
            matches_the_reference::<MaskedPartition, AuxiliaryRotation>(0, shapes);
            matches_the_reference::<MaskedPartition, BlockCycleRotation>(0, shapes);
        }
    }

    // Windows of one fresh element and of a few, of a block and of two, three
    // and six, so that windows end at every place a block can end and in the
    // middle of one, with blocks of the least length and of a fixed length.
    #[test]
    fn windows_of_every_awkward_size_match_the_reference() {
        matches_the_reference::<Windowed<0, 1>, ReversalRotation>(0, Windowed::<0, 1>::block);
        matches_the_reference::<Windowed<0, 7>, ReversalRotation>(0, Windowed::<0, 7>::block);
        matches_the_reference::<Windowed<0, 100>, TrinityRotation>(0, Windowed::<0, 100>::block);
        matches_the_reference::<Windowed<12, 1>, ReversalRotation>(0, Windowed::<12, 1>::block);
        matches_the_reference::<Windowed<12, 11>, ReversalRotation>(0, Windowed::<12, 11>::block);
        matches_the_reference::<Windowed<12, 12>, TrinityRotation>(0, Windowed::<12, 12>::block);
        matches_the_reference::<Windowed<12, 13>, ReversalRotation>(0, Windowed::<12, 13>::block);
        matches_the_reference::<Windowed<16, 16>, ReversalRotation>(0, Windowed::<16, 16>::block);
        matches_the_reference::<Windowed<16, 32>, TrinityRotation>(0, Windowed::<16, 32>::block);
        matches_the_reference::<Windowed<16, 48>, ReversalRotation>(0, Windowed::<16, 48>::block);
        matches_the_reference::<Windowed<16, 96>, AuxiliaryRotation>(0, Windowed::<16, 96>::block);
        matches_the_reference::<Windowed<16, 992>, ReversalRotation>(0, Windowed::<16, 992>::block);
    }

    // The scratch of the rotation, and one that holds the whole region, which a
    // buffered pass takes.
    #[test]
    fn a_longer_scratch_matches_the_reference() {
        for extra in [5, 40, 400] {
            matches_the_reference::<MaskedPartition, ReversalRotation>(extra, block_len);
            matches_the_reference::<Windowed<0, 7>, ReversalRotation>(extra, Windowed::<0, 7>::block);
        }
    }

    #[test]
    fn two_three_and_four_distinct_values_match_the_reference() {
        few_distinct_values_match_the_reference::<MaskedPartition, ReversalRotation>();
        few_distinct_values_match_the_reference::<MaskedPartition, AuxiliaryRotation>();
        few_distinct_values_match_the_reference::<Windowed<0, 7>, ReversalRotation>();
        few_distinct_values_match_the_reference::<Windowed<16, 32>, ReversalRotation>();
        few_distinct_values_match_the_reference::<EncodedPartition, ReversalRotation>();
    }

    // The block length leaves the cell that tells a block's type alone, and a
    // window of that block length fits the masks with two whole partial blocks.
    #[test]
    fn the_code_cells_leave_the_type_cell_alone_and_a_window_fits_its_masks() {
        code_cells_fit(block_len);
        code_cells_fit(tight);
        for n in (2..20_000).chain([1 << 20, 1 << 30, usize::MAX >> 1]) {
            let b = block_len(n);
            assert!(window(b) >= b && window(b) + 2 * b <= MASK_WORDS * 64, "n={n}: windows of {} after blocks of {b}", window(b));
        }
    }

    #[test]
    fn no_move_goes_unlogged() {
        replays_without_a_gap::<MaskedPartition>(block_len);
        replays_without_a_gap::<MaskedPartition>(tight);
        replays_without_a_gap::<Windowed<0, 7>>(tight);
    }

    #[test]
    fn the_moves_per_element_are_flat_in_the_size_and_below_the_rotation_partitions() {
        moves_are_flat_and_below_the_rotation_partition::<MaskedPartition>(block_len, 1 << 13);
    }

    #[test]
    fn every_shape_moves_a_few_times_per_element() { every_shape_is_cheap::<MaskedPartition>(block_len); }

    #[test]
    fn a_pass_with_nothing_to_move_costs_comparisons_only() {
        nothing_to_move_costs_comparisons_only::<MaskedPartition>(block_len);
        nothing_to_move_costs_comparisons_only::<MaskedPartition>(tight);
    }

    #[test]
    fn the_ends_that_are_in_place_cost_no_moves() { ends_in_place_cost_no_moves::<MaskedPartition>(); }

    #[test]
    fn an_empty_region_and_a_lone_element_cost_nothing() { empty_and_lone_regions_cost_nothing::<MaskedPartition>(); }

    // No scratch at all: the empty slice is enough for the reversal rotation,
    // whatever the size, since the partition keeps nothing but locals.
    #[test]
    fn an_empty_scratch_slice_is_enough() {
        let mut rng = Rng(0xE3);
        for n in [2usize, 50, 1000, 5000] {
            for shape in 0..SHAPES {
                let keys: Vec<u32> = types(shape, n, block_len(n), &mut rng).iter().map(|&f| key(f, &mut rng)).collect();
                let arr0 = items(&keys);
                let mut arr = arr0.clone();
                let split = MaskedPartition::partition::<ReversalRotation, _, _>(&mut arr, 0, n, item(PIVOT), &mut [], &mut sort_logger::NoOpLogger);
                assert_eq!((pairs(&arr), split), (reference(&arr0, |k| k < PIVOT), arr0.iter().filter(|x| x.key < PIVOT).count()), "n={n} shape={shape}");
            }
        }
        assert_eq!(MaskedPartition::scratch_size(1 << 20), 0);
    }

    // The masked phase one moves a front element once and a back element once,
    // where the buffered one moves a back element twice, so the whole partition
    // moves less.
    #[test]
    fn the_masked_partition_moves_less_than_the_encoded_one() {
        let (masked, _) = cost::<MaskedPartition, ReversalRotation>(1 << 16, 2, 1);
        let (encoded, _) = cost::<EncodedPartition, ReversalRotation>(1 << 16, 2, 1);
        assert!(masked < encoded, "{masked} moves per element against the encoded partition's {encoded}");
    }
}
