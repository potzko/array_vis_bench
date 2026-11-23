//! The masked stable partitions, linear time with no buffer of elements: the
//! whole blocks that `EncodedPartition` gathers through a buffer are made here
//! by permuting windows of the region, each window's classes held as a bit mask
//! in a few words. `permute_window` reads a window's mask and works out where
//! every element goes from popcounts of the mask below it, `Walk` applies those
//! places by cycles with one element carried in a local, `windows` slides the
//! window along the region, and `by_masks` is the whole partition, the `blocks`
//! module trimming the ends and ordering the blocks the windows leave. The two
//! walks are the two fillers: `MaskedPartition` works each place out by popcount
//! as it goes, with a second mask for the positions done, and
//! `MaskedTablePartition` fills a table of places first, which is faster and
//! takes more words. The masks, the counts and the walk's array are the fixed
//! size arrays a C version keeps on the stack, and here are words the driver of
//! the sort allocates once through the logger, so they are drawn and every write
//! to them is logged.

use core::marker::PhantomData;

use array_vis_bench_traits::role::{Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
use sort_logger::{split_sections, SortLogger};

use crate::blocks::{ceil_log2, finish, trim};
use crate::partition::buffered;

/// How the places of a window are applied to its elements.
pub trait Walk {
    /// The label of the partition that walks this way.
    const LABEL: &'static str;

    /// Words of the array this walk keeps beside the masks of a window, which
    /// `Masked` lays after them in its words.
    const WORDS: usize;

    /// Move the element at each position `p` of `arr[start..start + len]` to
    /// `place(p)`, which is a permutation of the positions, writing each element
    /// that moves once, by `write_data`. `words` is the walk's own `WORDS`, as
    /// they were left, written only by `write_data_u`.
    fn apply<T: Ord + Copy, U: ?Sized + SortLogger<T>, P: Fn(usize) -> usize>(arr: &mut [T], start: usize, len: usize, place: P, words: &mut [usize], log: &mut U);
}

// A mask holds 64 positions to a word, and a word is a `usize`.
const _: () = assert!(usize::BITS == 64, "a mask word holds 64 positions");

/// Each place is worked out by popcount as the cycle is walked, and a mask marks
/// the positions done: a few words and nothing else.
pub struct Popcount;
impl Walk for Popcount {
    const LABEL: &'static str = "masked<popcount>";
    const WORDS: usize = MASK_WORDS;

    // A cycle is followed with its element carried in a local: take the first
    // position not done whose place is elsewhere, write what it holds to its
    // place, carry the element that was there and write it to its place in turn,
    // until the cycle closes on the first position. The words are cleared first,
    // for the positions of the window.
    fn apply<T: Ord + Copy, U: ?Sized + SortLogger<T>, P: Fn(usize) -> usize>(arr: &mut [T], start: usize, len: usize, place: P, done: &mut [usize], log: &mut U) {
        for word in 0..len.div_ceil(64) { log.write_data_u(done, word, 0); }
        for first in 0..len {
            if done[first / 64] >> (first % 64) & 1 == 1 || place(first) == first { continue; }
            let (mut at, mut carried) = (first, arr[start + first]);
            loop {
                let marked = done[at / 64] | 1 << (at % 64);
                log.write_data_u(done, at / 64, marked);
                let to = place(at);
                let displaced = arr[start + to];
                log.write_data(arr, start + to, carried);
                if to == first { break; }
                (at, carried) = (to, displaced);
            }
        }
    }
}

/// The places of a window are filled into a table, a word a position, before any
/// element moves, and the cycles walk the table, marking a position done by
/// pointing its entry at itself. A word a position more than the walk by popcount
/// keeps, and a load of the table less in the walk.
pub struct Tabulated;
impl Walk for Tabulated {
    const LABEL: &'static str = "masked<table>";
    const WORDS: usize = MASK_WORDS * 64;

    fn apply<T: Ord + Copy, U: ?Sized + SortLogger<T>, P: Fn(usize) -> usize>(arr: &mut [T], start: usize, len: usize, place: P, table: &mut [usize], log: &mut U) {
        let dest = &mut table[..len];
        for p in 0..len { log.write_data_u(dest, p, place(p)); }
        for first in 0..len {
            if dest[first] == first { continue; }
            let (mut at, mut carried) = (first, arr[start + first]);
            loop {
                let to = dest[at];
                log.write_data_u(dest, at, at);
                let displaced = arr[start + to];
                log.write_data(arr, start + to, carried);
                if to == first { break; }
                (at, carried) = (to, displaced);
            }
        }
    }
}

/// Windows of the region permuted by bit masks into whole blocks, then the
/// blocks put in order as `EncodedPartition` does: `O(n)` moves and
/// comparisons, no scratch of its own, whichever way `K` walks the places.
pub struct Masked<K: Walk>(PhantomData<K>);

impl<K: Walk> Masked<K> {
    // The words of a window: the mask of back elements, the count of them below
    // each word, one more than the words of the mask, and the walk's own array.
    const SECTIONS: [usize; 3] = [MASK_WORDS, MASK_WORDS + 1, K::WORDS];
}

/// Masked, the places worked out by popcount during the walk.
pub type MaskedPartition = Masked<Popcount>;

/// Masked, the places tabulated before the walk.
pub type MaskedTablePartition = Masked<Tabulated>;

impl<K: Walk> StablePartition for Masked<K> {
    const NAME: &'static str = K::LABEL;
    const MOVES: Complexity = Complexity::N1;
    const SCRATCH_SECTIONS: &'static [usize] = &Self::SECTIONS;

    fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        words: &mut [usize],
        log: &mut U,
    ) -> usize {
        let b = block_len(hi - lo);
        by_masks::<K, R, T, U, _>(arr, lo, hi, b, window(b), &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), scratch, words, log)
    }

    fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        lo: usize,
        hi: usize,
        pivot: T,
        scratch: &mut [T],
        words: &mut [usize],
        log: &mut U,
    ) -> usize {
        let b = block_len(hi - lo);
        by_masks::<K, R, T, U, _>(arr, lo, hi, b, window(b), &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), scratch, words, log)
    }
}

// Words of a mask. A window, the partial blocks it starts with and the fresh
// elements it takes, is at most 64 times as many elements, 1024. A window holds
// bits and no element: the classes, and a count of the backs below each word.
// The words are `3 w + 1` by popcount and `66 w + 1` by table, `w` the words of
// the mask: 49 and 1057 words, 392 and 8456 bytes, at 16 words, where the buffer
// of `EncodedPartition` is 1280 bytes of `usize` at 2^20 elements, and it grows
// with the element type where these do not. Provisional: 16 words, the 1024 bits
// of the brief, is a guess at the knee of a curve that was flat from 16 to 64
// words on a loaded machine, and the combined measurement settles it.
const MASK_WORDS: usize = 16;

// Cells in a block: BLOCK_LOGS times the logarithm of `n`, rounded up, and at
// least two, as in `EncodedPartition`, and at most a quarter of a window, so that
// a window takes at least two blocks of fresh elements and the code cells of a
// block's rank, `ceil(log2(minority blocks))` of them, are fewer than its cells.
// Provisional: eight is the encoded partition's, and a block of that many
// logarithms sat on the flat part of the curve in the first trials, which a
// combined measurement is to confirm.
const BLOCK_LOGS: usize = 8;
const MAX_BLOCK: usize = MASK_WORDS * 64 / 4;

fn block_len(n: usize) -> usize { if n < 2 { 0 } else { (BLOCK_LOGS * ceil_log2(n)).clamp(2, MAX_BLOCK) } }

// Fresh elements a window takes on top of its partial blocks of fewer than `b`
// cells each, so that the whole window fits the masks.
fn window(b: usize) -> usize { MASK_WORDS * 64 - 2 * b }

// `front` says whether the element at an index is one of those to bring to the
// front, by one logged comparison against the pivot, `b` is the block length and
// `w` the fresh elements a window takes. The ends in place are left by `trim`,
// the blocks are made by `windows`, and put in order, with the loose elements
// placed, by `finish`.
fn by_masks<K: Walk, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    b: usize,
    w: usize,
    front: &F,
    scratch: &mut [T],
    words: &mut [usize],
    log: &mut U,
) -> usize {
    let (lo, hi) = match trim(arr, lo, hi, front, log) {
        Ok(region) => region,
        Err(split) => return split,
    };
    if hi - lo <= scratch.len() { return buffered(arr, lo, hi, front, scratch, log); }
    let (end, loose) = windows::<K, T, U, F>(arr, lo, hi, b, w, front, words, log);
    finish::<R, T, U, F>(arr, lo, end, loose, b, front, scratch, log)
}

// The region goes through in windows. Behind the window are whole blocks of one
// type each, `arr[lo..end]`, and then the partial blocks, `z` front elements
// followed by `h` back elements, which start the next window, with the next
// `w` elements still unread behind them. Each window is permuted into whole
// blocks and new partial blocks, so each element is compared once and moved
// about once, and nothing outside the masks is kept. Returns `end` and `z`, and
// the `h` back elements run to `hi`, as `finish` asks.
fn windows<K: Walk, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    b: usize,
    w: usize,
    front: &F,
    words: &mut [usize],
    log: &mut U,
) -> (usize, usize) {
    let (mut end, mut z, mut h) = (lo, 0, 0);
    while end + z + h < hi {
        let fresh = w.min(hi - end - z - h);
        let (whole, fronts, backs) = permute_window::<K, T, U, F>(arr, end, (z, h, fresh), b, front, words, log);
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
// The places are handed to the walk `K`, which applies them. Walking the cycles
// with each place worked out as it goes waits, at every step, on a load, a
// popcount and the selects before the next step can start; a table of the places,
// filled in a pass where no entry waits for another, takes that out of the walk,
// at a word a position more. Returns the cells in whole blocks, and the front and
// back elements left over. The mask, the counts and the walk's array are the
// sections of `words`, and every write to them is logged.
fn permute_window<K: Walk, T: Ord + Copy, U: ?Sized + SortLogger<T>, F: Fn(&mut U, &[T], usize) -> bool>(
    arr: &mut [T],
    start: usize,
    (z, h, fresh): (usize, usize, usize),
    b: usize,
    front: &F,
    words: &mut [usize],
    log: &mut U,
) -> (usize, usize, usize) {
    let len = z + h + fresh;
    assert!(len <= MASK_WORDS * 64, "a window of {len} elements does not fit {MASK_WORDS} words");
    let [back, below, rest] = split_sections(words, Masked::<K>::SECTIONS);
    // Bit `p` of the mask is set when the element at `p` is a back element. A
    // word is built in a register and stored once: a store to the same word for
    // each element is a chain through memory, and it cost four times as much.
    let used = len.div_ceil(64);
    for word in 0..used {
        let (from, to) = (word * 64, (word * 64 + 64).min(len));
        let held = (from.max(z)..to.min(z + h)).fold(0usize, |bits, p| bits | 1 << (p % 64));
        let bits = (from.max(z + h)..to).fold(held, |bits, p| bits | usize::from(!front(log, arr, start + p)) << (p % 64));
        log.write_data_u(back, word, bits);
    }
    // `below[i]` is the back elements in the words before word `i`.
    let mut backs = 0;
    for word in 0..used {
        log.write_data_u(below, word, backs);
        backs += back[word].count_ones() as usize;
    }
    log.write_data_u(below, used, backs);
    let fronts = len - backs;
    let (whole_fronts, whole_backs) = (fronts / b * b, backs / b * b);
    let (blocks, loose_fronts) = (whole_fronts + whole_backs, fronts - whole_fronts);
    let place = |p: usize| {
        let backs_before = below[p / 64] + (back[p / 64] & ((1usize << (p % 64)) - 1)).count_ones() as usize;
        let (rank, whole, first_loose, loose_base) = if back[p / 64] >> (p % 64) & 1 == 1 {
            (backs_before, whole_backs, whole_fronts, blocks + loose_fronts)
        } else {
            (p - backs_before, whole_fronts, 0, blocks)
        };
        if rank < whole { first_loose + rank } else { loose_base + rank - whole }
    };
    K::apply(arr, start, len, place, rest, log);
    (blocks, loose_fronts, backs - whole_backs)
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
    // place a block can, walked by `K`.
    struct Windowed<const B: usize, const W: usize, K = Popcount>(PhantomData<K>);
    impl<const B: usize, const W: usize, K> Windowed<B, W, K> {
        fn block(n: usize) -> usize { if B == 0 { tight(n) } else { B } }
    }
    impl<const B: usize, const W: usize, K: Walk> StablePartition for Windowed<B, W, K> {
        const NAME: &'static str = "masked, windowed";
        const MOVES: Complexity = Complexity::N1;
        const SCRATCH_SECTIONS: &'static [usize] = Masked::<K>::SCRATCH_SECTIONS;

        fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            pivot: T,
            scratch: &mut [T],
            words: &mut [usize],
            log: &mut U,
        ) -> usize {
            by_masks::<K, R, T, U, _>(arr, lo, hi, Self::block(hi - lo), W, &move |log: &mut U, arr: &[T], i| log.cmp_lt_data(arr, i, pivot), scratch, words, log)
        }

        fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            pivot: T,
            scratch: &mut [T],
            words: &mut [usize],
            log: &mut U,
        ) -> usize {
            by_masks::<K, R, T, U, _>(arr, lo, hi, Self::block(hi - lo), W, &move |log: &mut U, arr: &[T], i| !log.cmp_gt_data(arr, i, pivot), scratch, words, log)
        }
    }

    const BACK: usize = 1 << 20;

    // A window of `z` fronts, `h` backs and some unread elements comes out as
    // the whole blocks of fronts, the whole blocks of backs, the latest fronts
    // and the latest backs, each in order, with every unread element compared
    // once, every element that moves written once and none that stays touched.
    fn a_window_comes_out_as_whole_blocks_then_the_loose_elements<K: Walk>() {
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
                let got = permute_window::<K, _, _, _>(&mut arr, start, (z, h, fresh), b, &is_front, &mut words_of::<Masked<K>>(), &mut tally);
                let name = format!("{}: b={b} z={z} h={h} fresh={fresh}", K::LABEL);
                assert_eq!(&arr[start..start + before.len()], &expected[..], "{name}: the window");
                assert_eq!((&arr[..start], &arr[start + before.len()..]), (&vec![7; start][..], &[8, 9][..]), "{name}: outside the window");
                assert_eq!(got, (whole_fronts + whole_backs, fronts.len() - whole_fronts, backs.len() - whole_backs), "{name}: what is left over");
                let moved = before.iter().zip(&expected).filter(|(a, b)| a != b).count();
                assert_eq!((tally.moves, tally.cmps), (moved, fresh), "{name}: moves and comparisons");
            }
        }
    }

    #[test]
    fn a_window_comes_out_as_whole_blocks_then_the_loose_elements_by_either_walk() {
        a_window_comes_out_as_whole_blocks_then_the_loose_elements::<Popcount>();
        a_window_comes_out_as_whole_blocks_then_the_loose_elements::<Tabulated>();
    }

    // A walk puts every element where `place` says, writing each element that
    // moves once and logging every write, those to its words too.
    fn a_walk_writes_each_moved_element_once<K: Walk>() {
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
                let mut words = replay.create_aux_arr(K::WORDS);
                K::apply(&mut arr, start, len, |p| place[p], &mut words, &mut replay);
                let name = format!("{}: len={len} round={round}", K::LABEL);
                assert!((0..len).all(|p| arr[start + place[p]] == before[start + p]), "{name}: not placed");
                assert_eq!((&arr[..start], &arr[start + len..]), (&before[..start], &before[start + len..]), "{name}: outside");
                assert_eq!(replay.main(), &arr[..], "{name}: a move went unlogged");
                // The words are compared with their copy as they are freed.
                replay.free_aux_arr(&words);
                // Whatever the words held before, the walk does not read it.
                let mut again = before.clone();
                let mut tally = Tally::default();
                K::apply(&mut again, start, len, |p| place[p], &mut vec![usize::MAX; K::WORDS], &mut tally);
                assert_eq!(again, arr, "{name}: the walk read a word it had not written");
                assert_eq!(tally.moves, (0..len).filter(|&p| place[p] != p).count(), "{name}: moves");
            }
        }
    }

    #[test]
    fn a_walk_by_cycles_writes_each_moved_element_once() {
        a_walk_writes_each_moved_element_once::<Popcount>();
        a_walk_writes_each_moved_element_once::<Tabulated>();
    }

    #[test]
    fn the_masked_partitions_match_the_reference() {
        fn both<K: Walk>() {
            for shapes in [block_len, tight] {
                matches_the_reference::<Masked<K>, ReversalRotation>(0, shapes);
                matches_the_reference::<Masked<K>, TrinityRotation>(0, shapes);
                matches_the_reference::<Masked<K>, AuxiliaryRotation>(0, shapes);
                matches_the_reference::<Masked<K>, BlockCycleRotation>(0, shapes);
            }
        }
        both::<Popcount>();
        both::<Tabulated>();
    }

    // Windows of one fresh element and of a few, of a block and of two, three
    // and six, so that windows end at every place a block can end and in the
    // middle of one, with blocks of the least length and of a fixed length.
    #[test]
    fn windows_of_every_awkward_size_match_the_reference() {
        fn both<K: Walk>() {
            matches_the_reference::<Windowed<0, 1, K>, ReversalRotation>(0, Windowed::<0, 1, K>::block);
            matches_the_reference::<Windowed<0, 7, K>, ReversalRotation>(0, Windowed::<0, 7, K>::block);
            matches_the_reference::<Windowed<0, 100, K>, TrinityRotation>(0, Windowed::<0, 100, K>::block);
            matches_the_reference::<Windowed<12, 1, K>, ReversalRotation>(0, Windowed::<12, 1, K>::block);
            matches_the_reference::<Windowed<12, 11, K>, ReversalRotation>(0, Windowed::<12, 11, K>::block);
            matches_the_reference::<Windowed<12, 12, K>, TrinityRotation>(0, Windowed::<12, 12, K>::block);
            matches_the_reference::<Windowed<12, 13, K>, ReversalRotation>(0, Windowed::<12, 13, K>::block);
            matches_the_reference::<Windowed<16, 16, K>, ReversalRotation>(0, Windowed::<16, 16, K>::block);
            matches_the_reference::<Windowed<16, 32, K>, TrinityRotation>(0, Windowed::<16, 32, K>::block);
            matches_the_reference::<Windowed<16, 48, K>, ReversalRotation>(0, Windowed::<16, 48, K>::block);
            matches_the_reference::<Windowed<16, 96, K>, AuxiliaryRotation>(0, Windowed::<16, 96, K>::block);
            matches_the_reference::<Windowed<16, 992, K>, ReversalRotation>(0, Windowed::<16, 992, K>::block);
        }
        both::<Popcount>();
        both::<Tabulated>();
    }

    // The scratch of the rotation, and one that holds the whole region, which a
    // buffered pass takes.
    #[test]
    fn a_longer_scratch_matches_the_reference() {
        for extra in [5, 40, 400] {
            matches_the_reference::<MaskedPartition, ReversalRotation>(extra, block_len);
            matches_the_reference::<MaskedTablePartition, ReversalRotation>(extra, block_len);
            matches_the_reference::<Windowed<0, 7>, ReversalRotation>(extra, Windowed::<0, 7>::block);
        }
    }

    #[test]
    fn two_three_and_four_distinct_values_match_the_reference() {
        few_distinct_values_match_the_reference::<MaskedPartition, ReversalRotation>();
        few_distinct_values_match_the_reference::<MaskedPartition, AuxiliaryRotation>();
        few_distinct_values_match_the_reference::<MaskedTablePartition, ReversalRotation>();
        few_distinct_values_match_the_reference::<Windowed<0, 7>, ReversalRotation>();
        few_distinct_values_match_the_reference::<Windowed<16, 32, Tabulated>, ReversalRotation>();
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

    // What a failed test prints is the label the catalog gives the entry.
    #[test]
    fn the_names_are_the_catalog_labels() {
        assert_eq!(MaskedPartition::NAME, "masked<popcount>");
        assert_eq!(MaskedTablePartition::NAME, "masked<table>");
    }

    #[test]
    fn no_move_goes_unlogged() {
        replays_without_a_gap::<MaskedPartition>(block_len);
        replays_without_a_gap::<MaskedPartition>(tight);
        replays_without_a_gap::<MaskedTablePartition>(block_len);
        replays_without_a_gap::<MaskedTablePartition>(tight);
        replays_without_a_gap::<Windowed<0, 7>>(tight);
        replays_without_a_gap::<Windowed<0, 7, Tabulated>>(tight);
    }

    #[test]
    fn the_moves_per_element_are_flat_in_the_size_and_below_the_rotation_partitions() {
        moves_are_flat_and_below_the_rotation_partition::<MaskedPartition>(block_len, 1 << 13);
        moves_are_flat_and_below_the_rotation_partition::<MaskedTablePartition>(block_len, 1 << 13);
    }

    #[test]
    fn every_shape_moves_a_few_times_per_element() {
        every_shape_is_cheap::<MaskedPartition>(block_len);
        every_shape_is_cheap::<MaskedTablePartition>(block_len);
    }

    #[test]
    fn a_pass_with_nothing_to_move_costs_comparisons_only() {
        for shapes in [block_len, tight] {
            nothing_to_move_costs_comparisons_only::<MaskedPartition>(shapes);
            nothing_to_move_costs_comparisons_only::<MaskedTablePartition>(shapes);
        }
    }

    #[test]
    fn the_ends_that_are_in_place_cost_no_moves() {
        ends_in_place_cost_no_moves::<MaskedPartition>();
        ends_in_place_cost_no_moves::<MaskedTablePartition>();
    }

    #[test]
    fn an_empty_region_and_a_lone_element_cost_nothing() {
        empty_and_lone_regions_cost_nothing::<MaskedPartition>();
        empty_and_lone_regions_cost_nothing::<MaskedTablePartition>();
    }

    // No scratch of elements at all: the empty slice is enough for the reversal
    // rotation, whatever the size, since the partition keeps nothing but its words.
    #[test]
    fn an_empty_scratch_slice_is_enough() {
        fn both<K: Walk>() {
            let mut rng = Rng(0xE3);
            for n in [2usize, 50, 1000, 5000] {
                for shape in 0..SHAPES {
                    let keys: Vec<u32> = types(shape, n, block_len(n), &mut rng).iter().map(|&f| key(f, &mut rng)).collect();
                    let arr0 = items(&keys);
                    let mut arr = arr0.clone();
                    let split = Masked::<K>::partition::<ReversalRotation, _, _>(&mut arr, 0, n, item(PIVOT), &mut [], &mut words_of::<Masked<K>>(), &mut sort_logger::NoOpLogger);
                    assert_eq!((pairs(&arr), split), (reference(&arr0, |k| k < PIVOT), arr0.iter().filter(|x| x.key < PIVOT).count()), "{}: n={n} shape={shape}", K::LABEL);
                }
            }
            assert_eq!(Masked::<K>::scratch_size(1 << 20), 0);
        }
        both::<Popcount>();
        both::<Tabulated>();
    }

    // The masked phase one moves a front element once and a back element once,
    // where the buffered one moves a back element twice, so the whole partition
    // moves less.
    #[test]
    fn the_masked_partitions_move_less_than_the_encoded_one() {
        let (encoded, _) = cost::<EncodedPartition, ReversalRotation>(1 << 16, 2, 1);
        for (masked, _) in [cost::<MaskedPartition, ReversalRotation>(1 << 16, 2, 1), cost::<MaskedTablePartition, ReversalRotation>(1 << 16, 2, 1)] {
            assert!(masked < encoded, "{masked} moves per element against the encoded partition's {encoded}");
        }
    }

    // The words of a window: the mask of back elements, the count of them below
    // each word, and the walk's own array, the positions done by popcount or the
    // places of the table, a word a position. By popcount they take a few hundred
    // bytes, which is less than the encoded partition's buffer of `usize` elements
    // at 2^20, and the table of places takes more.
    #[test]
    fn the_walks_ask_for_the_words_of_their_arrays() {
        let encoded = size_of::<usize>() * EncodedPartition::scratch_size(1 << 20);
        assert_eq!((Popcount::WORDS, Tabulated::WORDS), (MASK_WORDS, MASK_WORDS * 64), "at {MASK_WORDS} words");
        assert_eq!(MaskedPartition::SCRATCH_SECTIONS, &[16, 17, 16], "the mask, the counts, the positions done");
        assert_eq!(MaskedTablePartition::SCRATCH_SECTIONS, &[16, 17, 1024], "the mask, the counts, the table");
        let bytes = |sections: &[usize]| size_of::<usize>() * sections.iter().sum::<usize>();
        assert_eq!((bytes(MaskedPartition::SCRATCH_SECTIONS), bytes(MaskedTablePartition::SCRATCH_SECTIONS)), (392, 8456));
        assert!(bytes(MaskedPartition::SCRATCH_SECTIONS) < encoded, "{} bytes against the encoded partition's {encoded}", bytes(MaskedPartition::SCRATCH_SECTIONS));
        assert!(bytes(MaskedTablePartition::SCRATCH_SECTIONS) > encoded);
        assert_eq!(<Windowed<0, 7, Tabulated> as StablePartition>::SCRATCH_SECTIONS, MaskedTablePartition::SCRATCH_SECTIONS);
    }

    // What the visualiser sees of a window of a partition by popcount: the mask
    // fills a word to each 64 comparisons of the sweep, the counts follow, and the
    // positions done are marked during the cycles, after the last count.
    #[test]
    fn the_mask_fills_during_the_sweep_and_the_done_marks_during_the_cycles() {
        use sort_logger::{SortLog, VisualizerLogger};
        let mut rng = Rng(0xD0E);
        let n = 3000;
        let mut arr: Vec<usize> = types(2, n, block_len(n), &mut rng).iter().zip(0..).map(|(&f, i)| if f { i } else { BACK + i }).collect();
        let mut log = VisualizerLogger { type_ghost: std::marker::PhantomData, log: Vec::new() };
        let mut words = log.create_aux_arr_split(MaskedPartition::SCRATCH_SECTIONS);
        let [back, below, done] = [0, MASK_WORDS, 2 * MASK_WORDS + 1].map(|at| words[at..].as_ptr() as usize);
        MaskedPartition::partition::<ReversalRotation, _, _>(&mut arr, 0, n, BACK, &mut [], &mut words, &mut log);
        // The kind of each event: a comparison, or a write to the mask, the counts or the done marks.
        let kinds: Vec<char> = log
            .log
            .iter()
            .filter_map(|e| match e {
                SortLog::CmpData { .. } => Some('c'),
                SortLog::WriteDataU { name, .. } => Some(if *name == back { 'm' } else if *name == below { 'n' } else if *name == done { 'd' } else { '?' }),
                _ => None,
            })
            .collect();
        assert!(!kinds.contains(&'?'), "a write to a word the sections do not name");
        let first = |kind: char| kinds.iter().position(|&k| k == kind).unwrap_or_else(|| panic!("no write of kind {kind}"));
        assert!(first('c') < first('m') && first('m') < first('n') && first('n') < first('d'), "the order of the first of each: {:?}", (first('c'), first('m'), first('n'), first('d')));
        // A word of the mask after the 64 comparisons of its positions: the gaps
        // between its writes, the first one holding the comparisons of the trim and
        // the last a partial word.
        let (mut gaps, mut since) = (Vec::new(), 0);
        for &kind in kinds.iter().take_while(|&&kind| kind != 'n') {
            if kind == 'c' { since += 1 } else { gaps.push(std::mem::take(&mut since)) }
        }
        assert!(gaps.len() >= 10, "{} words in the first window's mask", gaps.len());
        assert!(gaps[1..gaps.len() - 1].iter().all(|&gap| gap == 64), "comparisons between the words of the mask: {gaps:?}");
    }
}
