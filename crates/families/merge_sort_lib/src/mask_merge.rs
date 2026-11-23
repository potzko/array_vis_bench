//! `merge_window`, the stable merge of one window of two adjacent sorted runs
//! that fits a bit mask held in local variables. A scan that only compares
//! writes the output origin mask, then cycles of moves apply it, each element
//! written once. Every move and every comparison goes through the logger.

use sort_logger::SortLogger;

// Why it is stable. Merge the runs A and B the usual way, taking the next element
// of B only when it is below the next element of A, so an equal element of A goes
// first. Record the origin of each output position t: bit t of the mask M is set
// when position t takes an element of B. The elements of A leave in order and so
// do those of B, so position t takes the element `rank0(M, t)` of A when its bit
// is clear and the element `rank1(M, t)` of B when it is set, where `rank0` and
// `rank1` count the clear and the set bits below t. That is a permutation that
// puts every element where the stable merge does, and the ranks count output
// positions, so writing the permutation never changes them.
//
// The permutation is applied by cycles in the gather direction. Position t is
// filled from `source(t)`, and a cycle from a first position s lifts the element
// of s into a local, fills each position in turn from its source and ends when a
// source is s, where the local goes. Each position of a cycle is written once, and
// every source is read before it is written because it is written only when the
// walk reaches it. A second mask marks the positions walked.
//
// The window is trimmed, `arr[lo] > arr[mid]` and `arr[mid - 1] > arr[hi - 1]`:
// the first output is then an element of B and the last one of A, so B runs out
// first, the remainder of A is clear bits that need not be scanned, and no
// position is fixed, which makes every element of the window one move.

/// Merge `arr[lo..mid]` with `arr[mid..hi]`, both sorted, stably, when
/// `arr[lo] > arr[mid]`, `arr[mid - 1] > arr[hi - 1]` and the window `hi - lo`
/// is at most `64 * WORDS`. A window of `w` elements costs at most `w - 1`
/// comparisons and exactly `w` moves.
pub fn merge_window<const WORDS: usize, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    logger: &mut U,
) {
    let (left, window) = (mid - lo, hi - lo);
    assert!(lo < mid && mid < hi && window <= 64 * WORDS, "a window of {window} is not 1..={} cells", 64 * WORDS);

    // The origin mask, a word at a time, B running out ends the scan.
    let mut origin = [0u64; WORDS];
    let (mut a, mut b, mut t, mut word) = (lo, mid, 0usize, 0u64);
    while b < hi {
        let from_b = logger.cmp_lt(arr, b, a);
        word |= (from_b as u64) << (t & 63);
        b += from_b as usize;
        a += !from_b as usize;
        t += 1;
        if t & 63 == 0 {
            origin[(t - 1) >> 6] = word;
            word = 0;
        }
    }
    if t & 63 != 0 { origin[t >> 6] = word; }

    // The set bits below each word, so a rank is one popcount.
    let words = window.div_ceil(64);
    let mut below = [0usize; WORDS];
    for k in 1..words { below[k] = below[k - 1] + origin[k - 1].count_ones() as usize; }
    let source = |t: usize| {
        let (k, bit) = (t >> 6, t & 63);
        let ones = below[k] + (origin[k] & ((1u64 << bit) - 1)).count_ones() as usize;
        if (origin[k] >> bit) & 1 == 1 { left + ones } else { t - ones }
    };

    let mut walked = [0u64; WORDS];
    for k in 0..words {
        let valid = if (k + 1) * 64 <= window { !0u64 } else { (1u64 << (window & 63)) - 1 };
        loop {
            let free = !walked[k] & valid;
            if free == 0 { break; }
            let start = k * 64 + free.trailing_zeros() as usize;
            let carried = arr[lo + start];
            let mut at = start;
            loop {
                walked[at >> 6] |= 1 << (at & 63);
                let from = source(at);
                if from == start {
                    logger.write_data(arr, lo + at, carried);
                    break;
                }
                logger.write(arr, lo + at, lo + from);
                at = from;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{pairs, runs, sorted_keys, stable_merge, Item, Replay, Rng, Tally};
    use sort_logger::NoOpLogger;

    // Two sorted runs of random keys below `values`, trimmed: the first key of B is
    // below every key of A and the last of A above every key of B.
    fn trimmed_runs(rng: &mut Rng, la: usize, lb: usize, values: usize) -> (Vec<u32>, Vec<u32>) {
        let a = sorted_keys(rng, la - 1, values).into_iter().map(|k| k + 1).chain([values as u32 + 1]).collect();
        let b = [0].into_iter().chain(sorted_keys(rng, lb - 1, values).into_iter().map(|k| k + 1)).collect::<Vec<u32>>();
        (a, b)
    }

    fn merged_window<const WORDS: usize>(a: &[u32], b: &[u32]) -> (Vec<(u32, u32)>, Vec<(u32, u32)>, Tally) {
        let mut arr = runs(a, b);
        let want = stable_merge(&arr);
        let mut tally = Tally::default();
        let n = arr.len();
        merge_window::<WORDS, _, _>(&mut arr, 0, a.len(), n, &mut tally);
        (pairs(&arr), want, tally)
    }

    #[test]
    fn every_pair_of_run_lengths_up_to_the_mask_merges_stably() {
        let mut rng = Rng(0x3A5C);
        for la in 1..=40 {
            for lb in 1..=40 {
                for values in [1, 2, 5, 1000] {
                    let (a, b) = trimmed_runs(&mut rng, la, lb, values);
                    let (got, want, _) = merged_window::<2>(&a, &b);
                    assert_eq!(got, want, "la={la} lb={lb} values={values}");
                }
            }
        }
    }

    #[test]
    fn a_window_writes_every_element_once_and_compares_each_at_most_once() {
        let mut rng = Rng(0xBEE5);
        for (la, lb) in [(1, 1), (2, 1), (1, 2), (30, 34), (64, 64), (100, 28), (28, 100), (3, 125), (127, 1), (512, 512)] {
            for values in [2, 1000] {
                let (a, b) = trimmed_runs(&mut rng, la, lb, values);
                let (got, want, tally) = merged_window::<16>(&a, &b);
                assert_eq!(got, want, "la={la} lb={lb} values={values}");
                assert_eq!(tally.moves, la + lb, "la={la} lb={lb}: a move for each element");
                assert!(tally.cmps < la + lb, "la={la} lb={lb}: {} comparisons", tally.cmps);
            }
        }
    }

    #[test]
    fn windows_at_the_edges_of_a_word_and_of_the_mask_merge() {
        let mut rng = Rng(0x64);
        for w in [2, 63, 64, 65, 127, 128, 129, 191, 192, 193, 255, 256] {
            for la in [1, w / 2, w - 1] {
                let (a, b) = trimmed_runs(&mut rng, la, w - la, 6);
                let (got, want, tally) = merged_window::<4>(&a, &b);
                assert_eq!(got, want, "window {w} la={la}");
                assert_eq!(tally.moves, w, "window {w} la={la}");
            }
        }
    }

    #[test]
    fn a_window_of_alternating_elements_is_one_cycle_of_the_whole_window() {
        let (a, b): (Vec<u32>, Vec<u32>) = ((0..32).map(|i| 2 * i + 1).collect(), (0..32).map(|i| 2 * i).collect());
        let (got, want, tally) = merged_window::<1>(&a, &b);
        assert_eq!(got, want);
        assert_eq!(tally.moves, 64);
    }

    #[test]
    #[should_panic(expected = "is not 1..=")]
    fn a_window_past_the_mask_is_refused() {
        let (a, b): (Vec<u32>, Vec<u32>) = ((0..33).map(|i| 2 * i + 1).collect(), (0..32).map(|i| 2 * i).collect());
        merged_window::<1>(&a, &b);
    }

    // The logged moves are the whole story: replayed onto a copy they give the array.
    #[test]
    fn no_move_goes_unlogged() {
        let mut rng = Rng(0x10661);
        for (la, lb) in [(1, 1), (5, 9), (40, 24), (64, 64), (200, 300), (1, 700), (700, 1)] {
            let (a, b) = trimmed_runs(&mut rng, la, lb, 40);
            let mut arr: Vec<usize> = a.iter().chain(&b).map(|&k| k as usize).collect();
            let mut replay = Replay::of(&arr);
            merge_window::<16, _, _>(&mut arr, 0, la, la + lb, &mut replay);
            assert!(arr.windows(2).all(|w| w[0] <= w[1]), "la={la} lb={lb}");
            assert_eq!(replay.main(), &arr[..], "la={la} lb={lb}: a move went unlogged");
        }
    }

    #[test]
    fn a_window_inside_a_longer_array_leaves_the_rest_alone() {
        let mut rng = Rng(5);
        let (a, b) = trimmed_runs(&mut rng, 20, 25, 30);
        let mut arr: Vec<Item> = (0..7).map(|i| Item { key: 99, tag: 900 + i }).chain(runs(&a, &b)).chain((0..5).map(|i| Item { key: 98, tag: 800 + i })).collect();
        let want = stable_merge(&arr[7..52]);
        merge_window::<1, _, _>(&mut arr, 7, 27, 52, &mut NoOpLogger);
        assert_eq!(pairs(&arr[7..52]), want);
        assert!(arr[..7].iter().all(|x| x.key == 99 && x.tag >= 900) && arr[52..].iter().all(|x| x.key == 98 && x.tag >= 800));
    }
}
