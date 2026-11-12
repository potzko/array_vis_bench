//! Level sort, `LevelSort<P, R>`: the level ordered heap, built by stable
//! selection at the power of two boundaries, refined level by level by stable
//! selection of the median into a stable in place sort. `select` is the stable
//! quickselect over the three way partition, `build_levels` the heap, `refine`
//! the halving down to insertion sorted runs. The family is declared in
//! `level.spec` and built by `level.avbs`.

use core::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::role::keys::sort_region;
use array_vis_bench_traits::role::{Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

#[cfg(test)]
pub mod test_util;

// A run this short is left to the insertion sort.
const RUN: usize = 8;

/// `P` is the `StablePartition` the selections split with and `R` the `Rotation`
/// it rotates with, whose scratch is the one allocation of the sort.
pub struct LevelSort<P, R>(PhantomData<(P, R)>);

impl<P: StablePartition, R: Rotation> LevelSort<P, R> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], log: &mut U) {
        let n = arr.len();
        if n < 2 { return; }
        let cells = R::scratch_size(n);
        let mut scratch = if cells > 0 { log.create_aux_arr_t(cells) } else { Vec::new() };
        build_levels::<P, R, T, U>(arr, &mut scratch, log);
        let mut lo = 0;
        for hi in boundaries(n).chain([n]) {
            refine::<P, R, T, U>(arr, lo, hi, &mut scratch, log);
            lo = hi;
        }
        if cells > 0 { log.free_aux_arr_t(&scratch); }
    }
}

array_vis_bench_traits::direct_sort!(LevelSort<P, R>; P: StablePartition, R: Rotation);

// The selections halve, so the build and each level cost expected linear
// comparisons times the depth, and a quickselect is quadratic at worst.
impl<P: StablePartition, R: Rotation> HasTimeBounds for LevelSort<P, R> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N_LOG_N;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

impl<P: StablePartition, R: Rotation + HasSpace> HasSpace for LevelSort<P, R> {
    const SPACE: Complexity = R::SPACE;
}

// Every split keeps each side in order and a selection stops inside the
// pivot's equal group, so equal elements never cross each other.
impl<P: StablePartition, R: Rotation> HasStability for LevelSort<P, R> {
    const STABLE: bool = true;
}

/// The level boundaries `2^k - 1` inside an array of `n`, ascending.
pub fn boundaries(n: usize) -> impl Iterator<Item = usize> {
    (1..).map(|k| (1 << k) - 1).take_while(move |&b| b < n)
}

/// The level ordered heap: the smallest `2^k - 1` elements of `arr` at its
/// front for every boundary, from the largest down so that the prefixes halve,
/// equal elements in original order throughout.
pub fn build_levels<P: StablePartition, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    scratch: &mut [T],
    log: &mut U,
) {
    let n = arr.len();
    let mut hi = n;
    for b in boundaries(n).collect::<Vec<_>>().into_iter().rev() {
        select::<P, R, T, U>(arr, 0, hi, b, scratch, log);
        hi = b;
    }
}

/// Bring the `k` smallest elements of `arr[lo..hi]` to its front, ties to the
/// earlier, equal elements keeping their order in both parts: a quickselect over
/// the three way stable partition around a median of three, done as soon as the
/// boundary falls inside the pivot's equal group, which the stable split leaves
/// in order.
pub fn select<P: StablePartition, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    k: usize,
    scratch: &mut [T],
    log: &mut U,
) {
    let (mut lo, mut hi, mut k) = (lo, hi, k);
    while k > 0 && k < hi - lo {
        let pivot = arr[median_of_three(arr, lo, lo + (hi - lo) / 2, hi - 1, log)];
        let (equal, above) = P::partition_three_way::<R, T, U>(arr, lo, hi, pivot, scratch, log);
        if lo + k < equal {
            hi = equal;
        } else if lo + k > above {
            k -= above - lo;
            lo = above;
        } else {
            return;
        }
    }
}

// The index of the median of three cells, compared in place.
fn median_of_three<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], a: usize, b: usize, c: usize, log: &mut U) -> usize {
    let (low, high) = if log.cmp_lt(arr, b, a) { (b, a) } else { (a, b) };
    if log.cmp_lt(arr, c, low) {
        low
    } else if log.cmp_lt(arr, c, high) {
        c
    } else {
        high
    }
}

// Halve `arr[lo..hi]` by selection until a run fits the insertion sort.
fn refine<P: StablePartition, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    scratch: &mut [T],
    log: &mut U,
) {
    if hi - lo <= RUN {
        sort_region(arr, lo, hi, log);
        return;
    }
    let mid = (hi - lo) / 2;
    select::<P, R, T, U>(arr, lo, hi, mid, scratch, log);
    refine::<P, R, T, U>(arr, lo, lo + mid, scratch, log);
    refine::<P, R, T, U>(arr, lo + mid, hi, scratch, log);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{items, pairs, Item, Replay, Rng, Tally};
    use array_vis_bench_traits::DirectSort;
    use extraction_lib::{RotationPartition, SlidingPartition};
    use rotation_auxiliary::AuxiliaryRotation;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
    use sort_logger::NoOpLogger;
    use std::collections::{HashMap, HashSet};

    type Level<P, R> = LevelSort<P, R>;

    // The input shapes: every key repeated often, so that ties are the rule.
    fn input(kind: usize, n: usize, rng: &mut Rng) -> Vec<Item> {
        let keys: Vec<u32> = match kind {
            0 => (0..n).map(|_| rng.next(n.max(1) * 4) as u32).collect(),
            1 => (0..n as u32).collect(),
            2 => (0..n as u32).rev().collect(),
            3 => vec![5; n],
            4 => (0..n).map(|i| (i % 7) as u32).collect(),
            5 => (0..n).map(|_| rng.next(2) as u32).collect(),
            6 => (0..n).map(|_| rng.next(10) as u32).collect(),
            _ => (0..n).map(|_| rng.next(40) as u32).collect(),
        };
        items(&keys)
    }
    const KINDS: usize = 8;

    // The tags of the `k` smallest of `arr`, ties to the earlier.
    fn smallest(arr: &[Item], k: usize) -> HashSet<u32> {
        let mut ranked = arr.to_vec();
        ranked.sort_by_key(|x| x.key);
        ranked[..k].iter().map(|x| x.tag).collect()
    }

    // Equal keys appear in original order: a partition keeps the order within
    // each of its classes and a selection stops in an equal group, so an
    // element never passes an equal, while elements of different keys may.
    fn equals_in_order(arr: &[Item]) -> bool {
        let mut last = HashMap::new();
        arr.iter().all(|x| last.insert(x.key, x.tag).is_none_or(|t| t < x.tag))
    }

    // What a selection of `k` promises for `arr[lo..]`: the prefix is the `k`
    // smallest, ties to the earlier, and both parts keep equals in order.
    fn is_selected(arr: &[Item], k: usize, want: &HashSet<u32>) -> bool {
        let got: HashSet<u32> = arr[..k].iter().map(|x| x.tag).collect();
        got == *want && equals_in_order(&arr[..k]) && equals_in_order(&arr[k..])
    }

    #[test]
    fn the_boundaries_are_the_powers_of_two_less_one() {
        assert_eq!(boundaries(20).collect::<Vec<_>>(), [1, 3, 7, 15]);
        assert_eq!(boundaries(15).collect::<Vec<_>>(), [1, 3, 7]);
        assert_eq!(boundaries(1).count(), 0);
    }

    #[test]
    fn a_selection_brings_the_k_smallest_to_the_front_in_order() {
        let mut rng = Rng(0x5E1);
        for round in 0..500 {
            let n = 1 + rng.next(80);
            let kind = round % KINDS;
            let arr0 = input(kind, n, &mut rng);
            let (lo, k) = (rng.next(3).min(n - 1), rng.next(n + 1));
            let k = k.min(n - lo);
            let mut arr = arr0.clone();
            select::<RotationPartition, ReversalRotation, _, _>(&mut arr, lo, n, k, &mut [], &mut NoOpLogger);
            assert_eq!(pairs(&arr[..lo]), pairs(&arr0[..lo]), "round {round}: the prefix moved");
            assert!(is_selected(&arr[lo..], k, &smallest(&arr0[lo..], k)), "round {round}: kind {kind} n={n} lo={lo} k={k}: {:?}", pairs(&arr));
        }
    }

    // Each level holds the next rank interval, in original order among equals.
    #[test]
    fn the_levels_are_rank_intervals_in_original_order() {
        let mut rng = Rng(0x1E7E1);
        for n in [1usize, 2, 3, 7, 8, 15, 16, 100, 257, 1000] {
            for kind in 0..KINDS {
                let arr0 = input(kind, n, &mut rng);
                let mut arr = arr0.clone();
                build_levels::<RotationPartition, ReversalRotation, _, _>(&mut arr, &mut [], &mut NoOpLogger);
                for hi in boundaries(n) {
                    assert!(is_selected(&arr, hi, &smallest(&arr0, hi)), "kind {kind}, n={n}: the first {hi}");
                }
            }
        }
    }

    fn sorts_stably<S: DirectSort>(name: &str) {
        let mut rng = Rng(0x5EED);
        for n in [0usize, 1, 2, 3, 8, 9, 31, 32, 33, 64, 100, 257, 600, 1500, 3001] {
            for kind in 0..KINDS {
                let arr0 = input(kind, n, &mut rng);
                let mut arr = arr0.clone();
                S::sort(&mut arr, &mut NoOpLogger);
                let mut want = arr0.clone();
                want.sort_by_key(|x| x.key);
                assert_eq!(pairs(&arr), pairs(&want), "{name}: kind {kind}, n={n}");
            }
        }
    }

    #[test]
    fn sorts_stably_over_every_partition_and_rotation_under_test() {
        sorts_stably::<Level<RotationPartition, ReversalRotation>>("rotation, reversal");
        sorts_stably::<Level<RotationPartition, TrinityRotation>>("rotation, trinity");
        sorts_stably::<Level<RotationPartition, AuxiliaryRotation>>("rotation, auxiliary");
        sorts_stably::<Level<SlidingPartition, ReversalRotation>>("sliding, reversal");
    }

    #[test]
    fn no_move_goes_unlogged() {
        let mut rng = Rng(0x10661);
        for n in [32usize, 33, 100, 257, 600, 1500] {
            for values in [1, 2, 5, 12, 40, 100_000] {
                let mut arr: Vec<usize> = (0..n).map(|_| rng.next(values)).collect();
                let mut replay = Replay::of(&arr);
                Level::<RotationPartition, ReversalRotation>::sort(&mut arr, &mut replay);
                assert!(arr.windows(2).all(|w| w[0] <= w[1]), "not sorted, n={n} values={values}");
                assert_eq!(replay.main(), &arr[..], "a move went unlogged, n={n} values={values}");
            }
        }
    }

    // The selections cost a few times the range in comparisons and `n log n`
    // moves each, over `log n` levels of halving.
    #[test]
    fn costs_n_log_n_comparisons_and_n_log_squared_moves() {
        let n = 4096;
        let mut rng = Rng(0xC0FFEE);
        let mut arr: Vec<usize> = (0..n).map(|_| rng.next(1 << 30)).collect();
        let mut tally = Tally::default();
        Level::<RotationPartition, ReversalRotation>::sort(&mut arr, &mut tally);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        assert!(tally.cmps < 8 * n * 12, "comparisons {}", tally.cmps);
        assert!(tally.moves < 4 * n * 12 * 12, "moves {}", tally.moves);
    }

    #[test]
    fn the_bounds_follow_from_the_rotation() {
        type Sort = Level<RotationPartition, ReversalRotation>;
        assert_eq!(<Sort as HasTimeBounds>::WORST, Complexity::N_SQUARED);
        assert_eq!(<Sort as HasTimeBounds>::AVERAGE, Complexity::N_LOG_N);
        assert!(<Sort as HasSpace>::SPACE.is_in_place());
        assert!(<Sort as HasStability>::STABLE);
        assert!(!<Level<RotationPartition, AuxiliaryRotation> as HasSpace>::SPACE.is_in_place());
    }
}
