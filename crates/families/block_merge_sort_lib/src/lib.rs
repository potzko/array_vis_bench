//! Block merge sort, `BlockMergeSort<Buffer, Keys>`: a stable bottom up merge
//! sort whose merges use a small buffer and a key region instead of an `O(n)`
//! scratch array. `sort` carves distinct elements out of the array, sorts the
//! rest by merging runs with `roll_merge`, then sorts the carved elements back in.
//! `stash` is where a local merge parks its left run and `rolling` is the block
//! merge. The family is declared in `block_merge.spec` and built by
//! `block_merge.avbs`.

use core::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::role::keys::{first_not_below, sort_region};
use array_vis_bench_traits::role::{Auxiliary, Keys, Rotation};
use array_vis_bench_traits::Complexity;
use rotation_reversal::ReversalRotation;
use scratchspace_lib::bufferless_merge;
use sort_logger::{RegionKind, SortLogger};

pub mod rolling;
pub mod stash;

use rolling::roll_merge;
use stash::Stash;

#[cfg(test)]
mod test_util;

/// `B` is the `Auxiliary` buffer that backs the local merges and `K` the `Keys`
/// that tag the blocks. An in array buffer with in array keys is WikiSort, a
/// fixed external buffer with in array keys is GrailSort with a buffer.
pub struct BlockMergeSort<B, K>(PhantomData<(B, K)>);

// Runs of RUN elements are insertion sorted before the first merge, and an array
// shorter than SMALL is one insertion sort.
const RUN: usize = 8;
const SMALL: usize = 32;
// Fewer distinct elements than this cannot key a block merge.
const MIN_KEYS: usize = 4;

// A merge costs `O(n)` when blocks roll and a buffer holds each, and the rotation
// merge's `O(n log n)` when either is missing. `log n` levels of merges, so the
// bound of the weaker of the two axes times `log n`.
impl<B: Auxiliary, K: Keys> HasTimeBounds for BlockMergeSort<B, K> {
    const WORST: Complexity =
        Complexity::product(Complexity::LOG_N, Complexity::sum(K::MERGE_WORST, B::MERGE_WORST));
    // A sorted array is one insertion sort per run and a comparison per merge.
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity =
        Complexity::product(Complexity::LOG_N, Complexity::sum(K::MERGE_WORST, B::MERGE_WORST));
}

impl<B: Auxiliary, K: Keys> HasSpace for BlockMergeSort<B, K> {
    const SPACE: Complexity = Complexity::sum(B::MERGE_SPACE, K::MERGE_SPACE);
}

// Every local merge and the final redistribution keep an equal key of the left
// run ahead, the carved elements are the first occurrence of their values, and
// the run sort is an insertion sort.
impl<B: Auxiliary, K: Keys> HasStability for BlockMergeSort<B, K> {
    const STABLE: bool = true;
}

impl<B: Auxiliary, K: Keys> BlockMergeSort<B, K> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], log: &mut U) {
        let n = arr.len();
        if n < SMALL {
            sort_region(arr, 0, n, log);
            return;
        }
        // The keys come first in the carved region and the buffer behind them. A
        // block of `isqrt(n)` elements gives `n / isqrt(n)` blocks to tag.
        let (want_keys, want_buffer) = (K::key_count(n / n.isqrt() + 2), B::internal_size(n));
        let found = K::extract(arr, 0, n, want_keys + want_buffer, log);
        let (keys, buffer) = settle(found, want_keys, want_buffer, log);
        if keys > 0 { log.hold_region(arr, 0, keys, RegionKind::Keys); }
        if buffer > 0 { log.hold_region(arr, keys, buffer, RegionKind::Buffer); }

        let external = B::scratch_size(n);
        // A buffer the array's blocks outgrow still serves the runs that fit it, and
        // the block merges past that rotate their local merges.
        if keys > 0 && external > 0 && external < n.isqrt() {
            log.mark(format!(
                "block sort: the buffer of {external} cannot hold a block of {}, the block merges rotate",
                n.isqrt()
            ));
        }
        let mut ext = if external > 0 { log.create_aux_arr_t(external) } else { Vec::new() };
        if external > 0 { log.hold_region(&ext, 0, external, RegionKind::Buffer); }
        let mut stash = match (external > 0, buffer > 0) {
            (true, _) => Stash::External(&mut ext),
            (false, true) => Stash::Internal { at: keys, cap: buffer },
            (false, false) => Stash::None,
        };
        merge_levels::<K, T, U>(arr, found, &mut stash, keys, log);
        drop(stash);
        if external > 0 {
            log.release_region(&ext, 0, external, RegionKind::Buffer);
            log.free_aux_arr_t(&ext);
        }

        // The keys come back in order, the buffer is scrambled by the swaps, and
        // the carved elements are one sorted run, which is slid into the rest.
        K::restore(arr, 0, keys, log);
        if keys > 0 { log.release_region(arr, 0, keys, RegionKind::Keys); }
        sort_region(arr, keys, keys + buffer, log);
        if buffer > 0 { log.release_region(arr, keys, buffer, RegionKind::Buffer); }
        slide_merge(arr, 0, found, n, log);
    }
}

array_vis_bench_traits::direct_sort!(BlockMergeSort<B, K>; B: Auxiliary, K: Keys);

// The ladder. An array with too few distinct elements cannot give all the keys
// and the buffer, so the sort gives up the buffer first, then keys: it keeps what
// `found` covers and logs what it dropped. With fewer than MIN_KEYS distinct
// elements it merges by rotation alone. Nothing is capped without a mark.
fn settle<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    found: usize,
    keys: usize,
    buffer: usize,
    log: &mut U,
) -> (usize, usize) {
    let kept = if found >= keys + buffer {
        (keys, buffer)
    } else if keys == 0 {
        (0, 0)
    } else if found >= keys {
        (keys, 0)
    } else if found >= MIN_KEYS {
        (found, 0)
    } else {
        (0, 0)
    };
    if kept != (keys, buffer) {
        log.mark(format!(
            "block sort: {found} distinct elements for {keys} keys and a buffer of {buffer}, \
             keeping {} keys and a buffer of {}, the local merges that lack one rotate",
            kept.0, kept.1
        ));
    }
    kept
}

// Insertion sorted runs of RUN from `from`, then merges of adjacent runs at
// doubling widths.
fn merge_levels<K: Keys, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    from: usize,
    stash: &mut Stash<T>,
    keys: usize,
    log: &mut U,
) {
    let n = arr.len();
    for start in (from..n).step_by(RUN) { sort_region(arr, start, (start + RUN).min(n), log); }
    let mut width = RUN;
    while from + width < n {
        let mut lo = from;
        while lo + width < n {
            let mid = lo + width;
            merge_pair::<K, T, U>(arr, lo, mid, (mid + width).min(n), stash, keys, log);
            lo = mid + width;
        }
        width *= 2;
    }
}

// One merge of `arr[lo..mid]` with `arr[mid..hi]`. A left run the stash holds is
// parked and merged whole. A longer one rolls blocks if there are keys, in the
// stash if a block fits it and by rotation if not, and with no keys it is the
// rotation merge.
fn merge_pair<K: Keys, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    stash: &mut Stash<T>,
    keys: usize,
    log: &mut U,
) {
    if log.cmp_le(arr, mid - 1, mid) { return; }
    let (la, lb) = (mid - lo, hi - mid);
    if la <= stash.capacity() {
        stash.park(arr, lo, la, log);
        stash.merge(arr, lo, la, lb, log);
    } else if let Some(s) = block_size(la, keys) {
        let mut nowhere = Stash::None;
        let blocks_fit = s <= stash.capacity();
        roll_merge::<K, T, U>(arr, lo, mid, hi, s, 0, if blocks_fit { stash } else { &mut nowhere }, log);
    } else {
        bufferless_merge(arr, lo, mid, hi, log);
    }
}

// The block size of a merge with `la` left elements and `keys` keys: about the
// square root, larger when the keys do not reach, and none under four elements.
fn block_size(la: usize, keys: usize) -> Option<usize> {
    if keys == 0 { return None; }
    let s = la.isqrt().max(la.div_ceil(keys));
    (s >= 2).then_some(s)
}

// Merge the small sorted run `arr[lo..mid]` into `arr[mid..hi]`, an equal key of
// the small run first. The run slides right: each of its elements in turn finds
// the elements of the big run below it by one binary search and the run rotates
// past them, so an element of the big run moves once and the run costs `O(m^2)`.
fn slide_merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    log: &mut U,
) {
    let (mut start, mut end) = (lo, mid);
    while start < end && end < hi {
        let cut = first_not_below(arr, end, hi, start, log);
        if cut > end {
            ReversalRotation::rotate(&mut arr[start..cut], end - start, &mut [], log);
            start += cut - end;
            end = cut;
        }
        start += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{pairs, Item, Marks, Regions, Replay, Rng, Tally};
    use sort_logger::RegionKind;
    use scratchspace_lib::{FixedBuffer, FullBuffer, HalfBuffer, InArrayBuffer, InArrayKeys, NoBuffer, NoKeys};
    use sort_logger::NoOpLogger;

    // The shapes of input a sort sees, each with every key repeated `values` times
    // or so; `tag` records the original position.
    fn input(kind: usize, n: usize, rng: &mut Rng) -> Vec<Item> {
        let keys: Vec<u32> = match kind {
            0 => (0..n).map(|_| rng.next(n.max(1) * 4) as u32).collect(),
            1 => (0..n as u32).collect(),
            2 => (0..n as u32).rev().collect(),
            3 => vec![5; n],
            4 => (0..n).map(|i| (i % 7) as u32).collect(),
            5 => (0..n).map(|i| (i % 50) as u32).collect(),
            6 => (0..n).map(|i| (i.min(n - i - 1)) as u32).collect(),
            7 => (0..n).map(|_| rng.next(2) as u32).collect(),
            8 => (0..n).map(|_| rng.next(3) as u32).collect(),
            9 => (0..n).map(|_| rng.next(10) as u32).collect(),
            10 => (0..n).map(|_| rng.next(40) as u32).collect(),
            _ => (0..n).map(|i| if i < 20 { 1 } else { rng.next(n * 4) as u32 }).collect(),
        };
        keys.into_iter().zip(0u32..).map(|(key, tag)| Item { key, tag }).collect()
    }
    const KINDS: usize = 12;

    fn sorts_stably<B: Auxiliary, K: Keys>(name: &str) {
        let mut rng = Rng(0x5EED);
        for n in [0usize, 1, 2, 3, 8, 31, 32, 33, 64, 100, 257, 600, 1500, 3001] {
            for kind in 0..KINDS {
                let arr0 = input(kind, n, &mut rng);
                let mut arr = arr0.clone();
                BlockMergeSort::<B, K>::sort(&mut arr, &mut NoOpLogger);
                let mut want = arr0.clone();
                want.sort_by_key(|x| x.key);
                assert_eq!(pairs(&arr), pairs(&want), "{name}: kind {kind}, n={n}");
            }
        }
    }

    #[test]
    fn the_wiki_shape_sorts_stably() { sorts_stably::<InArrayBuffer, InArrayKeys>("in array, in array"); }
    #[test]
    fn the_grail_shape_with_a_buffer_sorts_stably() { sorts_stably::<FixedBuffer, InArrayKeys>("fixed, in array"); }
    #[test]
    fn the_shape_with_keys_and_no_buffer_sorts_stably() { sorts_stably::<NoBuffer, InArrayKeys>("none, in array"); }
    #[test]
    fn an_in_array_buffer_without_keys_sorts_stably() { sorts_stably::<InArrayBuffer, NoKeys>("in array, none"); }
    #[test]
    fn a_fixed_buffer_without_keys_sorts_stably() { sorts_stably::<FixedBuffer, NoKeys>("fixed, none"); }
    #[test]
    fn the_shape_with_nothing_sorts_stably() { sorts_stably::<NoBuffer, NoKeys>("none, none"); }
    #[test]
    fn a_buffer_the_size_of_the_array_sorts_stably() {
        sorts_stably::<FullBuffer, NoKeys>("full, none");
        sorts_stably::<FullBuffer, InArrayKeys>("full, in array");
        sorts_stably::<HalfBuffer, InArrayKeys>("half, in array");
    }

    // Every move is logged: replaying the events on a copy gives the sorted array.
    fn moves_are_all_logged<B: Auxiliary, K: Keys>(name: &str) {
        let mut rng = Rng(0x10661);
        for n in [32usize, 33, 100, 257, 600, 1500, 4000] {
            for values in [1, 2, 5, 12, 40, 100_000] {
                let mut arr: Vec<usize> = (0..n).map(|_| rng.next(values)).collect();
                let mut replay = Replay::of(&arr);
                BlockMergeSort::<B, K>::sort(&mut arr, &mut replay);
                assert!(arr.windows(2).all(|w| w[0] <= w[1]), "{name}: not sorted, n={n} values={values}");
                assert_eq!(replay.main(), &arr[..], "{name}: a move went unlogged, n={n} values={values}");
            }
        }
    }

    #[test]
    fn no_shape_moves_an_element_without_logging_it() {
        moves_are_all_logged::<InArrayBuffer, InArrayKeys>("in array, in array");
        moves_are_all_logged::<FixedBuffer, InArrayKeys>("fixed, in array");
        moves_are_all_logged::<NoBuffer, InArrayKeys>("none, in array");
        moves_are_all_logged::<InArrayBuffer, NoKeys>("in array, none");
        moves_are_all_logged::<FixedBuffer, NoKeys>("fixed, none");
        moves_are_all_logged::<NoBuffer, NoKeys>("none, none");
    }

    // The marks a sort logged on `arr`.
    fn marks<B: Auxiliary, K: Keys>(arr: &[Item]) -> Vec<String> {
        let mut arr = arr.to_vec();
        let mut log = Marks::default();
        BlockMergeSort::<B, K>::sort(&mut arr, &mut log);
        assert!(arr.windows(2).all(|w| w[0].key <= w[1].key), "not sorted");
        log.0
    }

    #[test]
    fn enough_distinct_elements_log_nothing() {
        let mut rng = Rng(3);
        let arr = input(0, 600, &mut rng);
        assert!(marks::<InArrayBuffer, InArrayKeys>(&arr).is_empty());
        assert!(marks::<FixedBuffer, InArrayKeys>(&arr).is_empty());
        assert!(marks::<NoBuffer, NoKeys>(&arr).is_empty());
    }

    // The blocks of an array of 270000 are 519 elements, more than a fixed buffer holds.
    #[test]
    fn blocks_that_outgrow_the_fixed_buffer_are_logged() {
        let arr = input(0, 270_000, &mut Rng(5));
        let marked = marks::<FixedBuffer, InArrayKeys>(&arr);
        assert_eq!(marked.len(), 1, "{marked:?}");
        assert!(marked[0].contains("the buffer of 512 cannot hold a block of 519"), "{marked:?}");
        // blocks that fit it, or no blocks, or a buffer of the array's own, are not
        assert!(marks::<FixedBuffer, InArrayKeys>(&input(0, 100_000, &mut Rng(6))).is_empty());
        assert!(marks::<FixedBuffer, NoKeys>(&arr).is_empty());
        assert!(marks::<InArrayBuffer, InArrayKeys>(&arr).is_empty());
    }

    #[test]
    fn a_shortage_of_distinct_elements_is_logged_and_never_silent() {
        let mut rng = Rng(4);
        // 600 elements want 27 keys and a buffer of 24: ten values give the keys
        // fewer, three give no keys at all
        for kind in [9, 8, 7, 3] {
            let arr = input(kind, 600, &mut rng);
            let marked = marks::<InArrayBuffer, InArrayKeys>(&arr);
            assert_eq!(marked.len(), 1, "kind {kind}: {marked:?}");
            assert!(marked[0].starts_with("block sort: "), "{marked:?}");
            assert!(!marks::<FixedBuffer, InArrayKeys>(&arr).is_empty(), "kind {kind} starves the keys of a fixed buffer");
            assert!(!marks::<InArrayBuffer, NoKeys>(&arr).is_empty(), "kind {kind} starves the buffer");
        }
        // the buffer alone is dropped when only the keys fit
        let arr = input(10, 600, &mut rng);
        let marked = marks::<InArrayBuffer, InArrayKeys>(&arr);
        assert_eq!(marked.len(), 1, "{marked:?}");
        assert!(marked[0].contains("keeping 27 keys and a buffer of 0"), "{marked:?}");
        // the shapes that ask for nothing have nothing to drop
        assert!(marks::<NoBuffer, NoKeys>(&input(3, 600, &mut rng)).is_empty());
        assert!(marks::<FixedBuffer, NoKeys>(&input(3, 600, &mut rng)).is_empty());
    }

    // The region events of a sort of `arr`, without the array names but told apart
    // by whether they name the array being sorted.
    fn regions<B: Auxiliary, K: Keys>(arr: &[Item]) -> Vec<(bool, usize, usize, RegionKind, bool)> {
        let mut arr = arr.to_vec();
        let mut log = Regions::default();
        let name = arr.as_ptr() as usize;
        BlockMergeSort::<B, K>::sort(&mut arr, &mut log);
        log.0.into_iter().map(|(n, start, len, kind, held)| (n == name, start, len, kind, held)).collect()
    }

    #[test]
    fn the_keys_and_the_buffer_are_held_while_they_work_and_released_after() {
        use RegionKind::{Buffer, Keys};
        let arr = input(0, 600, &mut Rng(21));
        // 600 elements carve 27 keys and 24 buffer cells, the keys first
        assert_eq!(
            regions::<InArrayBuffer, InArrayKeys>(&arr),
            [(true, 0, 27, Keys, true), (true, 27, 24, Buffer, true), (true, 0, 27, Keys, false), (true, 27, 24, Buffer, false)]
        );
        // an external buffer is its own array, held around the merges
        assert_eq!(
            regions::<FixedBuffer, InArrayKeys>(&arr),
            [(true, 0, 27, Keys, true), (false, 0, 512, Buffer, true), (false, 0, 512, Buffer, false), (true, 0, 27, Keys, false)]
        );
        assert_eq!(regions::<InArrayBuffer, NoKeys>(&arr), [(true, 0, 24, Buffer, true), (true, 0, 24, Buffer, false)]);
        assert!(regions::<NoBuffer, NoKeys>(&arr).is_empty());
    }

    #[test]
    fn only_the_regions_that_survive_the_ladder_are_held() {
        use RegionKind::Keys;
        // ten distinct values keep ten keys and no buffer, one value keeps nothing
        let few = input(9, 600, &mut Rng(22));
        assert_eq!(regions::<InArrayBuffer, InArrayKeys>(&few), [(true, 0, 10, Keys, true), (true, 0, 10, Keys, false)]);
        let one = input(3, 600, &mut Rng(23));
        assert!(regions::<InArrayBuffer, InArrayKeys>(&one).is_empty());
    }

    // A merge sort that costs `n log n` moves grows 22 times for 16 times the input.
    fn moves<B: Auxiliary, K: Keys>(n: usize) -> usize {
        let mut rng = Rng(9);
        let mut arr: Vec<usize> = (0..n).map(|_| rng.next(n * 8)).collect();
        let mut tally = Tally::default();
        BlockMergeSort::<B, K>::sort(&mut arr, &mut tally);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        tally.moves
    }

    #[test]
    fn the_wiki_and_grail_shapes_move_elements_n_log_n_times() {
        for (small, large) in [
            (moves::<InArrayBuffer, InArrayKeys>(1024), moves::<InArrayBuffer, InArrayKeys>(16384)),
            (moves::<FixedBuffer, InArrayKeys>(1024), moves::<FixedBuffer, InArrayKeys>(16384)),
        ] {
            assert!(large < 30 * small, "moves {small} -> {large}");
        }
    }

    #[test]
    fn the_rotation_shapes_move_elements_n_log_squared_times() {
        let (small, large) = (moves::<NoBuffer, NoKeys>(1024), moves::<NoBuffer, NoKeys>(16384));
        assert!(large < 60 * small, "moves {small} -> {large}");
    }

    #[test]
    fn slide_merge_puts_a_small_run_into_a_big_one() {
        let mut rng = Rng(13);
        for _ in 0..300 {
            let (m, rest) = (rng.next(8), rng.next(40));
            let values = 1 + rng.next(10);
            let small = crate::test_util::sorted_run(&mut rng, m, values, 0);
            let big = crate::test_util::sorted_run(&mut rng, rest, values, 100);
            let mut arr: Vec<Item> = small.iter().chain(&big).copied().collect();
            slide_merge(&mut arr, 0, m, m + rest, &mut NoOpLogger);
            assert_eq!(pairs(&arr), crate::test_util::merged(&small, &big));
        }
    }

    #[test]
    fn the_bounds_follow_from_the_axes() {
        type Wiki = BlockMergeSort<InArrayBuffer, InArrayKeys>;
        assert_eq!(<Wiki as HasTimeBounds>::WORST, Complexity::N_LOG_N);
        assert_eq!(<Wiki as HasTimeBounds>::BEST, Complexity::N1);
        assert!(<Wiki as HasSpace>::SPACE.is_in_place());
        assert!(<Wiki as HasStability>::STABLE);
        // a missing buffer or missing keys leaves a merge the rotation merge's cost
        assert_eq!(<BlockMergeSort<FixedBuffer, InArrayKeys> as HasTimeBounds>::WORST, Complexity::N_LOG_SQUARED);
        assert_eq!(<BlockMergeSort<NoBuffer, InArrayKeys> as HasTimeBounds>::WORST, Complexity::N_LOG_SQUARED);
        assert_eq!(<BlockMergeSort<InArrayBuffer, NoKeys> as HasTimeBounds>::WORST, Complexity::N_LOG_SQUARED);
        // a buffer that grows with the array is not in place
        assert!(!<BlockMergeSort<FullBuffer, NoKeys> as HasSpace>::SPACE.is_in_place());
    }

    // The role reaches the inherent sort.
    #[test]
    fn the_direct_sort_role_sorts() {
        use array_vis_bench_traits::DirectSort;
        let mut arr: Vec<usize> = (0..100).rev().collect();
        <BlockMergeSort<InArrayBuffer, InArrayKeys> as DirectSort>::sort(&mut arr, &mut NoOpLogger);
        assert_eq!(arr, (0..100).collect::<Vec<_>>());
    }
}
