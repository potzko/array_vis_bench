//! `MaskRotationMerge`, a stable in place merge with no scratch of its own: it
//! trims the runs to the window that needs merging and, when the window fits a
//! bit mask held in local variables, `merge_window` merges it by an origin mask
//! and cycles; a longer window is split by rotations, as symMerge splits, until
//! the pieces fit, or handed whole to the smaller side merge when `SPLIT` is off.
//! Every move and every comparison goes through the logger.

use core::marker::PhantomData;

use super::rotation_merge::SmallerSideRotationMerge;
use super::utils::{lower_bound, upper_bound};
use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::{Complexity, Rotation};
use sort_logger::SortLogger;

use array_vis_bench_traits::role::merge::{MergeStrategy, RotationMerge};

/// Merge by the mask where the window fits `64 * WORDS` positions, and by
/// symMerge's split over the rotation `R` where it does not. The mask is two
/// arrays of `WORDS` words and a third of their running popcounts, all on the
/// stack, so the merge needs the rotation's scratch and no other. With `SPLIT`
/// off a window past the cap is not split: it goes whole to the smaller side merge,
/// which never uses the mask, the fallback the split is to be measured against.
/// The catalog takes the defaults, 16 words and the split. The 16 is provisional,
/// the choice of the cap waits for the measurement of the combined branch.
pub struct MaskRotationMerge<R: Rotation, const WORDS: usize = 16, const SPLIT: bool = true> {
    _phantom: PhantomData<R>,
}

/// The mask merge with its defaults, over whichever rotation it is given.
pub struct MaskMerge;

impl MergeStrategy for MaskMerge {
    type With<R: Rotation> = MaskRotationMerge<R>;
}

impl<R: Rotation, const WORDS: usize, const SPLIT: bool> RotationMerge for MaskRotationMerge<R, WORDS, SPLIT> {
    #[inline]
    fn scratch_size(n: usize) -> usize {
        R::scratch_size(n)
    }

    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], mid: usize, scratch: &mut [T], _words: &mut [usize], logger: &mut U) {
        let n = arr.len();
        merge_trimmed::<WORDS, SPLIT, R, T, U>(arr, 0, mid, n, scratch, logger);
    }
}

// The window is merged by the mask, one move an element, and a window split `d` times
// costs the `d` rotations of its pieces on top. A piece is cut at the middle of its
// shorter run, so a merge of two runs of `n / 2` splits `log (n / cap)` times and costs
// `O(n log (n / cap))` moves where symMerge costs `O(n log n)`. A lopsided merge is cut
// more times than that and costs up to the `O(n log n)` of symMerge, and a merge of
// runs in order costs the one comparison that says so. The fallback is the smaller side
// merge past the cap and has its bounds.
impl<R: Rotation, const WORDS: usize, const SPLIT: bool> HasTimeBounds for MaskRotationMerge<R, WORDS, SPLIT> {
    const WORST: Complexity = if SPLIT { Complexity::N_LOG_N } else { Complexity::N_LOG_SQUARED };
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Self::WORST;
}
// The mask is a constant number of words, so the space is the rotation's.
impl<R: Rotation + HasSpace, const WORDS: usize, const SPLIT: bool> HasSpace for MaskRotationMerge<R, WORDS, SPLIT> {
    const SPACE: Complexity = R::SPACE;
}
impl<R: Rotation, const WORDS: usize, const SPLIT: bool> HasStability for MaskRotationMerge<R, WORDS, SPLIT> {
    /// The mask is the origin of a stable merge, and the split is symMerge's, which
    /// cuts at a `lower_bound` or an `upper_bound` so equal keys keep their order.
    const STABLE: bool = true;
}

// Merge `arr[lo..mid]` with `arr[mid..hi]`. The elements of A not above B[0] and of B
// not below A[last] are in place, and both ends are found by binary search. What is
// left has `arr[lo] > arr[mid]` and `arr[mid - 1] > arr[hi - 1]`, which is the window
// `merge_window` takes, and it is cut at the middle of its shorter run when it is too
// long: the pivot splits the other run by a bound, one rotation puts the two lower
// pieces together and the two upper ones together, and each pair is merged the same way.
// Without `SPLIT` the long window is the smaller side merge's.
fn merge_trimmed<const WORDS: usize, const SPLIT: bool, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    mid: usize,
    hi: usize,
    scratch: &mut [T],
    logger: &mut U,
) {
    if lo >= mid || mid >= hi || logger.cmp_le(arr, mid - 1, mid) { return; }
    let lo = upper_bound(arr, lo, mid, arr[mid], logger);
    let hi = lower_bound(arr, mid, hi, arr[mid - 1], logger);
    let (left, right) = (mid - lo, hi - mid);
    if left + right <= 64 * WORDS {
        merge_window::<WORDS, T, U>(arr, lo, mid, hi, logger);
    } else if !SPLIT {
        SmallerSideRotationMerge::<R>::merge(&mut arr[lo..hi], mid - lo, scratch, &mut [], logger);
    } else if left <= right {
        let p = lo + left / 2;
        let q = lower_bound(arr, mid, hi, arr[p], logger);
        R::rotate(&mut arr[p..q], mid - p, scratch, logger);
        let cut = p + (q - mid);
        merge_trimmed::<WORDS, SPLIT, R, T, U>(arr, lo, p, cut, scratch, logger);
        merge_trimmed::<WORDS, SPLIT, R, T, U>(arr, cut, q, hi, scratch, logger);
    } else {
        let p = mid + right / 2;
        let q = upper_bound(arr, lo, mid, arr[p], logger);
        R::rotate(&mut arr[q..p + 1], mid - q, scratch, logger);
        let cut = q + (p + 1 - mid);
        merge_trimmed::<WORDS, SPLIT, R, T, U>(arr, lo, q, cut, scratch, logger);
        merge_trimmed::<WORDS, SPLIT, R, T, U>(arr, cut, p + 1, hi, scratch, logger);
    }
}

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
    use crate::rotation_merge::{SmallerSideMerge, SmallerSideRotationMerge};
    use crate::test_util::{pairs, runs, sorted_keys, stable_merge, words_of, Item, Replay, Rng, Tally};
    use rotation_auxiliary::AuxiliaryRotation;
    use rotation_block_cycle::BlockCycleRotation;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
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

    // The two runs `a` and `b` in one array, tagged in order, merged by `M` and compared
    // with a stable sort, with the scratch `M` asks for.
    fn assert_merges<M: RotationMerge>(a: &[u32], b: &[u32], label: &str) {
        let mut arr = runs(a, b);
        let want = stable_merge(&arr);
        let mut scratch = vec![Item { key: 0, tag: 0 }; M::scratch_size(arr.len())];
        M::merge(&mut arr, a.len(), &mut scratch, &mut words_of::<M>(), &mut NoOpLogger);
        assert_eq!(pairs(&arr), want, "{label}: la={} lb={}", a.len(), b.len());
    }

    // `n` sorted random keys in `from..to`.
    fn between(rng: &mut Rng, n: usize, from: u32, to: u32) -> Vec<u32> {
        sorted_keys(rng, n, (to - from) as usize).into_iter().map(|k| k + from).collect()
    }

    // The shapes two runs overlap in: disjoint either way round, one inside the other
    // either way round, interleaved, all equal, a few values and a spread of values,
    // and every key of one run in the other.
    const SHAPES: usize = 9;
    fn shape(kind: usize, rng: &mut Rng, la: usize, lb: usize) -> (Vec<u32>, Vec<u32>) {
        let outside = |rng: &mut Rng, n: usize| [between(rng, n / 2, 0, 1000), between(rng, n - n / 2, 2000, 3000)].concat();
        match kind {
            0 => ((0..la as u32).collect(), (la as u32..(la + lb) as u32).collect()),
            1 => ((lb as u32..(la + lb) as u32).collect(), (0..lb as u32).collect()),
            2 => (between(rng, la, 1000, 2000), outside(rng, lb)),
            3 => (outside(rng, la), between(rng, lb, 1000, 2000)),
            4 => ((0..la as u32).map(|i| 2 * i + 1).collect(), (0..lb as u32).map(|i| 2 * i).collect()),
            5 => (vec![7; la], vec![7; lb]),
            6 => (sorted_keys(rng, la, 3), sorted_keys(rng, lb, 3)),
            7 => (sorted_keys(rng, la, 10 * (la + lb)), sorted_keys(rng, lb, 10 * (la + lb))),
            8 => ((0..la as u32).map(|i| i / 2).collect(), (0..lb as u32).map(|i| i / 2).collect()),
            _ => unreachable!(),
        }
    }

    // Every pair of run lengths to 40 under a cap of one word, so windows past 64 split,
    // and under the default cap.
    fn every_pair_of_lengths<M: RotationMerge>(label: &str) {
        let mut rng = Rng(0x57AB1E);
        for la in 0..=40 {
            for lb in 0..=40 {
                for values in [1, 3, 1000] {
                    assert_merges::<M>(&sorted_keys(&mut rng, la, values), &sorted_keys(&mut rng, lb, values), label);
                }
            }
        }
    }

    #[test]
    fn every_pair_of_run_lengths_merges_stably() {
        every_pair_of_lengths::<MaskRotationMerge<ReversalRotation, 1>>("reversal, one word");
        every_pair_of_lengths::<MaskRotationMerge<ReversalRotation>>("reversal, default cap");
    }

    // Runs past the cap, at it and one past it, in every shape, under several rotations
    // and caps.
    fn every_shape<M: RotationMerge>(label: &str) {
        let mut rng = Rng(0xA11CE);
        for (la, lb) in [(1, 300), (300, 1), (32, 32), (33, 32), (32, 33), (64, 65), (100, 300), (512, 512), (513, 511), (1024, 1), (1, 1024), (1500, 700), (2000, 2100), (50, 5000), (5000, 50)] {
            for kind in 0..SHAPES {
                let (a, b) = shape(kind, &mut rng, la, lb);
                assert_merges::<M>(&a, &b, &format!("{label}, shape {kind}"));
            }
        }
    }

    #[test]
    fn every_shape_of_overlap_merges_stably() {
        every_shape::<MaskRotationMerge<ReversalRotation, 1>>("reversal, 1");
        every_shape::<MaskRotationMerge<ReversalRotation, 2>>("reversal, 2");
        every_shape::<MaskRotationMerge<ReversalRotation>>("reversal, 16");
        every_shape::<MaskRotationMerge<ReversalRotation, 64>>("reversal, 64");
    }

    #[test]
    fn several_rotations_split_the_window() {
        every_shape::<MaskRotationMerge<TrinityRotation, 1>>("trinity");
        every_shape::<MaskRotationMerge<BlockCycleRotation, 2>>("block cycle");
        every_shape::<MaskRotationMerge<AuxiliaryRotation, 1>>("auxiliary");
    }

    // A rotation that counts how often it is asked.
    thread_local! { static ROTATIONS: core::cell::Cell<usize> = const { core::cell::Cell::new(0) }; }
    struct Counting;
    impl Rotation for Counting {
        const NAME: &'static str = "counting";
        fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], split_ind: usize, scratch: &mut [T], logger: &mut U) {
            ROTATIONS.with(|c| c.set(c.get() + 1));
            ReversalRotation::rotate(arr, split_ind, scratch, logger);
        }
    }

    // A window of `w` elements, `a` odd keys against `b` even ones, so every element moves.
    fn window_of(w: usize) -> (Vec<u32>, Vec<u32>) {
        ((0..w.div_ceil(2) as u32).map(|i| 2 * i + 1).collect(), (0..(w / 2) as u32).map(|i| 2 * i).collect())
    }

    // Under the cap the window is one mask and no rotation, past it the window splits.
    fn at_the_cap<const WORDS: usize>() {
        let cap = 64 * WORDS;
        for w in [cap - 2, cap - 1, cap, cap + 1, cap + 2, 2 * cap, 2 * cap + 1] {
            let (a, b) = window_of(w);
            ROTATIONS.with(|c| c.set(0));
            let mut arr = runs(&a, &b);
            let want = stable_merge(&arr);
            let mut tally = Tally::default();
            <MaskRotationMerge<Counting, WORDS> as RotationMerge>::merge(&mut arr, a.len(), &mut [], &mut words_of::<MaskRotationMerge<Counting, WORDS>>(), &mut tally);
            assert_eq!(pairs(&arr), want, "cap {cap} window {w}");
            let rotations = ROTATIONS.with(|c| c.get());
            if w <= cap {
                assert_eq!((rotations, tally.moves), (0, w), "cap {cap} window {w}: one mask, a move each");
            } else {
                assert!(rotations > 0 && tally.moves > w, "cap {cap} window {w}: the window splits");
            }
        }
    }

    #[test]
    fn a_window_at_the_cap_is_one_mask_and_one_past_it_splits() {
        at_the_cap::<1>();
        at_the_cap::<2>();
        at_the_cap::<16>();
    }

    #[test]
    fn what_the_trim_leaves_is_the_window() {
        // Twenty elements of A that are not above B[0] and twenty of B that are not
        // below A[last] stay, and the 64 between them are one mask.
        let (a, b) = window_of(64);
        let a: Vec<u32> = (0..20).map(|_| 0).chain(a.iter().map(|k| k + 1)).collect();
        let b: Vec<u32> = b.iter().map(|k| k + 1).chain((0..20).map(|_| 1000)).collect();
        let mut arr = runs(&a, &b);
        let want = stable_merge(&arr);
        let mut tally = Tally::default();
        <MaskRotationMerge<Counting, 1> as RotationMerge>::merge(&mut arr, a.len(), &mut [], &mut words_of::<MaskRotationMerge<Counting, 1>>(), &mut tally);
        assert_eq!(pairs(&arr), want);
        assert_eq!(tally.moves, 64);
    }

    // The window a merge leaves after the trim, by the definition: the elements of A above
    // B[0] and the elements of B below A[last], none when the runs are in order.
    fn window_by_definition(a: &[u32], b: &[u32]) -> usize {
        match (a.last(), b.first()) {
            (Some(&last), Some(&first)) if last > first => a.iter().filter(|&&x| x > first).count() + b.iter().filter(|&&y| y < last).count(),
            _ => 0,
        }
    }

    #[test]
    fn a_window_that_fits_moves_each_element_once() {
        let mut rng = Rng(0xFEED);
        for _ in 0..600 {
            let (la, lb) = (rng.next(600), rng.next(600));
            let values = [2, 50, 100_000][rng.next(3)];
            let (a, b) = (sorted_keys(&mut rng, la, values), sorted_keys(&mut rng, lb, values));
            let window = window_by_definition(&a, &b);
            if window > 1024 { continue; }
            let mut arr = runs(&a, &b);
            let mut tally = Tally::default();
            <MaskRotationMerge<Counting, 16> as RotationMerge>::merge(&mut arr, la, &mut [], &mut words_of::<MaskRotationMerge<Counting, 16>>(), &mut tally);
            assert_eq!(tally.moves, window, "la={la} lb={lb} values={values}");
            let searches = 3 * ((la + lb).ilog2() as usize + 2);
            assert!(tally.cmps <= window + searches, "la={la} lb={lb}: {} comparisons for a window of {window}", tally.cmps);
        }
    }

    // No move goes unlogged, and the scratch of the rotation is registered with the
    // logger as a sort registers it.
    fn replayed<M: RotationMerge>(label: &str) {
        let mut rng = Rng(0x10661);
        for _ in 0..200 {
            let (la, lb) = (rng.next(300), rng.next(300));
            let values = 1 + rng.next(60);
            let (a, b) = (sorted_keys(&mut rng, la, values), sorted_keys(&mut rng, lb, 40));
            let mut arr: Vec<usize> = a.iter().chain(&b).map(|&k| k as usize).collect();
            let mut replay = Replay::of(&arr);
            let mut scratch = vec![usize::MAX; M::scratch_size(arr.len())];
            replay.log_aux_arr_t(&scratch);
            M::merge(&mut arr, la, &mut scratch, &mut words_of::<M>(), &mut replay);
            assert!(arr.windows(2).all(|w| w[0] <= w[1]), "{label}: la={la} lb={lb}");
            assert_eq!(replay.main(), &arr[..], "{label}: la={la} lb={lb}: a move went unlogged");
        }
    }

    #[test]
    fn no_move_of_the_merge_goes_unlogged() {
        replayed::<MaskRotationMerge<ReversalRotation, 1>>("reversal");
        replayed::<MaskRotationMerge<TrinityRotation, 2>>("trinity");
        replayed::<MaskRotationMerge<AuxiliaryRotation, 1>>("auxiliary");
    }

    // The moves of a merge of two random runs of `n / 2`, under a cap of one word.
    fn moves_per_element<M: RotationMerge>(n: usize) -> f64 {
        let mut rng = Rng(11 + n as u64);
        let (a, b) = (sorted_keys(&mut rng, n / 2, 1 << 30), sorted_keys(&mut rng, n / 2, 1 << 30));
        let mut arr = runs(&a, &b);
        let mut scratch = vec![Item { key: 0, tag: 0 }; M::scratch_size(n)];
        let mut tally = Tally::default();
        M::merge(&mut arr, n / 2, &mut scratch, &mut words_of::<M>(), &mut tally);
        tally.moves as f64 / n as f64
    }

    #[test]
    fn splitting_costs_half_a_move_a_level_over_the_cap_not_over_the_run() {
        // A level rotates the half of a window that lies between the pivots, which is
        // half a move an element under the reversal rotation, and the mask is one more.
        let per_element: Vec<f64> = (10..=18).step_by(2).map(|k| moves_per_element::<MaskRotationMerge<ReversalRotation, 1>>(1 << k)).collect();
        for pair in per_element.windows(2) {
            assert!((0.8..1.3).contains(&(pair[1] - pair[0])), "two levels more for a run four times as long: {per_element:?}");
        }
        assert!(per_element[4] < 0.5 * (18.0 - 6.0) + 1.5, "{per_element:?}");
        // Over the default cap of 1024 positions there are six levels fewer, and
        // symMerge splits down to the single element.
        let mask = moves_per_element::<MaskRotationMerge<ReversalRotation>>(1 << 18);
        let smaller_side = moves_per_element::<SmallerSideRotationMerge<ReversalRotation>>(1 << 18);
        assert!(mask * 1.4 < smaller_side, "mask {mask} against smaller side {smaller_side}");
    }

    #[test]
    fn the_merge_asks_for_the_rotation_scratch_and_is_stable() {
        assert_eq!(<MaskRotationMerge<ReversalRotation> as RotationMerge>::scratch_size(1000), 0);
        assert_eq!(<MaskRotationMerge<TrinityRotation> as RotationMerge>::scratch_size(1000), TrinityRotation::scratch_size(1000));
        assert_eq!(<MaskRotationMerge<AuxiliaryRotation> as RotationMerge>::scratch_size(1000), 500);
        assert!(<MaskRotationMerge<ReversalRotation> as HasStability>::STABLE);
    }

    // The fallback, symMerge for a window past the cap, is the same merge for the
    // windows that fit and a stable merge for the rest.
    #[test]
    fn the_fallback_merges_stably() {
        every_pair_of_lengths::<MaskRotationMerge<ReversalRotation, 1, false>>("reversal, one word, fallback");
        every_shape::<MaskRotationMerge<ReversalRotation, 1, false>>("reversal, 1, fallback");
        every_shape::<MaskRotationMerge<TrinityRotation, 2, false>>("trinity, 2, fallback");
        every_shape::<MaskRotationMerge<AuxiliaryRotation, 16, false>>("auxiliary, 16, fallback");
        replayed::<MaskRotationMerge<ReversalRotation, 1, false>>("reversal, fallback");
    }

    #[test]
    fn a_window_that_fits_is_one_mask_under_the_fallback_too() {
        let (a, b) = window_of(128);
        ROTATIONS.with(|c| c.set(0));
        let mut arr = runs(&a, &b);
        let mut tally = Tally::default();
        <MaskRotationMerge<Counting, 2, false> as RotationMerge>::merge(&mut arr, a.len(), &mut [], &mut words_of::<MaskRotationMerge<Counting, 2, false>>(), &mut tally);
        assert_eq!((ROTATIONS.with(|c| c.get()), tally.moves), (0, 128));
    }

    #[test]
    fn the_split_moves_fewer_cells_than_the_fallback_past_the_cap() {
        for k in [12, 14, 16] {
            let split = moves_per_element::<MaskRotationMerge<ReversalRotation, 1>>(1 << k);
            let fallback = moves_per_element::<MaskRotationMerge<ReversalRotation, 1, false>>(1 << k);
            assert!(split < fallback, "2^{k}: split {split}, fallback {fallback}");
        }
    }

    #[test]
    fn the_fallback_has_symmerges_bounds_and_the_split_the_mask_s() {
        assert_eq!(<MaskRotationMerge<ReversalRotation> as HasTimeBounds>::WORST, Complexity::N_LOG_N);
        assert_eq!(<MaskRotationMerge<ReversalRotation, 16, false> as HasTimeBounds>::WORST, Complexity::N_LOG_SQUARED);
        assert!(<MaskRotationMerge<ReversalRotation, 16, false> as HasStability>::STABLE);
    }

    // A strategy is the merge it names under the rotation it is given, and merges as it does.
    #[test]
    fn a_strategy_is_its_merge_over_the_rotation_it_is_given() {
        use core::any::type_name;
        assert_eq!(
            type_name::<<MaskMerge as MergeStrategy>::With<TrinityRotation>>(),
            type_name::<MaskRotationMerge<TrinityRotation>>()
        );
        assert_eq!(
            type_name::<<SmallerSideMerge as MergeStrategy>::With<TrinityRotation>>(),
            type_name::<SmallerSideRotationMerge<TrinityRotation>>()
        );
        every_shape::<<MaskMerge as MergeStrategy>::With<BlockCycleRotation>>("mask strategy, block cycle");
        every_shape::<<SmallerSideMerge as MergeStrategy>::With<AuxiliaryRotation>>("smaller side strategy, auxiliary");
    }
}
