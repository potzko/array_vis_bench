//! Stable quick sort, `StableQuickSort<P, R>`: a quick sort whose split is a
//! stable three way partition `P`, rotating with `R`, so that equal elements
//! never cross each other and the sort needs no tag and no index. `sort` is the
//! driver, `pivot_of` reads the pivot value off a median of three, a ninther
//! past `NINTHER` elements, and `sort_range` splits, recurses into the smaller
//! side and loops on the larger. The family is declared in `stable_quick.spec`
//! and built by `stable_quick.avbs`.

use core::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::role::keys::sort_region;
use array_vis_bench_traits::role::pivot::median_index;
use array_vis_bench_traits::role::{Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

#[cfg(test)]
pub mod test_util;

/// `P` is the `StablePartition` that splits a region three ways and `R` the
/// `Rotation` it rotates with. The scratch the two share is the one allocation
/// of the sort.
pub struct StableQuickSort<P, R>(PhantomData<(P, R)>);

impl<P: StablePartition, R: Rotation> StableQuickSort<P, R> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], log: &mut U) {
        let n = arr.len();
        if n < 2 { return; }
        let cells = R::scratch_size(n).max(P::scratch_size(n));
        let mut scratch = if cells > 0 { log.create_aux_arr_t(cells) } else { Vec::new() };
        sort_range::<P, R, T, U>(arr, 0, n, &mut scratch, log);
        if cells > 0 { log.free_aux_arr_t(&scratch); }
    }
}

array_vis_bench_traits::direct_sort!(StableQuickSort<P, R>; P: StablePartition, R: Rotation);

// Without a pivot that is certain to split evenly the depth, and with it the
// work, is `n` at worst, and one pass of the partition per level: its moves times
// `n` in the worst case and times `log n` on a pivot that splits well enough on
// average. A partition keeps what it does not move in order, so there is nothing
// to add for the elements the split passes over.
impl<P: StablePartition, R: Rotation> HasTimeBounds for StableQuickSort<P, R> {
    const WORST: Complexity = Complexity::product(Complexity::N1, P::MOVES);
    const BEST: Complexity = Complexity::product(Complexity::LOG_N, P::MOVES);
    const AVERAGE: Complexity = Self::BEST;
}

// The stack of the recursion into the smaller side, and the rotation's scratch;
// the partition's own cells are a logarithm of the array at most.
impl<P: StablePartition, R: Rotation + HasSpace> HasSpace for StableQuickSort<P, R> {
    const SPACE: Complexity = Complexity::sum(Complexity::LOG_N, R::SPACE);
}

// A stable partition keeps the order of equals on both sides and the equal part
// is in order, and the insertion sort below the cutoff is stable.
impl<P: StablePartition, R: Rotation> HasStability for StableQuickSort<P, R> {
    const STABLE: bool = true;
}

// A region this short is left to the insertion sort.
const SMALL: usize = 16;
// From this many elements the pivot is the median of three medians of three.
const NINTHER: usize = 128;

fn pivot_of<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], lo: usize, hi: usize, log: &mut U) -> T {
    let n = hi - lo;
    let at = |k: usize| lo + k * (n - 1) / 8;
    let index = if n < NINTHER {
        median_index(arr, log, lo, lo + n / 2, hi - 1)
    } else {
        let thirds = [(0, 1, 2), (3, 4, 5), (6, 7, 8)].map(|(a, b, c)| median_index(arr, log, at(a), at(b), at(c)));
        median_index(arr, log, thirds[0], thirds[1], thirds[2])
    };
    arr[index]
}

// Split `arr[lo..hi]` around the pivot into below, equal and above, each in
// order, sort the below and the above, and leave the equal part where it is. The
// smaller of the two sides is the recursion and the larger the next turn of the
// loop, so the stack is `log n` frames deep whatever the pivots do.
fn sort_range<P: StablePartition, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    scratch: &mut [T],
    log: &mut U,
) {
    let (mut lo, mut hi) = (lo, hi);
    while hi - lo > SMALL {
        let pivot = pivot_of(arr, lo, hi, log);
        let (equal, above) = P::partition_three_way::<R, T, U>(arr, lo, hi, pivot, scratch, log);
        if equal - lo < hi - above {
            sort_range::<P, R, T, U>(arr, lo, equal, scratch, log);
            lo = above;
        } else {
            sort_range::<P, R, T, U>(arr, above, hi, scratch, log);
            hi = equal;
        }
    }
    sort_region(arr, lo, hi, log);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{items, pairs, Item, Replay, Rng, Tally};
    use array_vis_bench_traits::DirectSort;
    use extraction_lib::{EncodedPartition, RotationPartition, SlidingPartition};
    use rotation_auxiliary::AuxiliaryRotation;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
    use sort_logger::NoOpLogger;

    // The input shapes: a tie is the rule in some, a sorted run in others, and
    // the ones that sink a plain first element pivot.
    fn input(kind: usize, n: usize, rng: &mut Rng) -> Vec<Item> {
        let keys: Vec<u32> = match kind {
            0 => (0..n).map(|_| rng.next(n.max(1) * 4) as u32).collect(),
            1 => (0..n as u32).collect(),
            2 => (0..n as u32).rev().collect(),
            3 => vec![5; n],
            4 => (0..n).map(|i| (i % 7) as u32).collect(),
            5 => (0..n).map(|_| rng.next(2) as u32).collect(),
            6 => (0..n).map(|_| rng.next(10) as u32).collect(),
            7 => (0..n).map(|_| rng.next(40) as u32).collect(),
            8 => (0..n).map(|i| i.min(n - i - 1) as u32).collect(),
            9 => (0..n).map(|i| (i % 50) as u32).collect(),
            10 => (0..n).map(|i| if i % 2 == 0 { i as u32 } else { (n - i) as u32 }).collect(),
            _ => (0..n).map(|i| if i < n / 2 { 0 } else { rng.next(n.max(1)) as u32 }).collect(),
        };
        items(&keys)
    }
    const KINDS: usize = 12;

    fn sorts_stably<S: DirectSort>(name: &str) {
        let mut rng = Rng(0x5EED);
        for n in [0usize, 1, 2, 3, 8, 15, 16, 17, 31, 32, 33, 64, 100, 127, 128, 129, 257, 600, 1500, 3001, 9000] {
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

    // Every partition filler, so every way a region is split.
    #[test]
    fn every_partition_sorts_stably() {
        sorts_stably::<StableQuickSort<RotationPartition, ReversalRotation>>("rotation, reversal");
        sorts_stably::<StableQuickSort<SlidingPartition, ReversalRotation>>("sliding, reversal");
        sorts_stably::<StableQuickSort<EncodedPartition, ReversalRotation>>("encoded, reversal");
    }

    // The rotation is `R`'s, and trinity and the auxiliary rotation run through
    // the scratch the sort allocates once.
    #[test]
    fn any_rotation_sorts_stably() {
        sorts_stably::<StableQuickSort<EncodedPartition, TrinityRotation>>("encoded, trinity");
        sorts_stably::<StableQuickSort<EncodedPartition, AuxiliaryRotation>>("encoded, auxiliary");
        sorts_stably::<StableQuickSort<RotationPartition, TrinityRotation>>("rotation, trinity");
        sorts_stably::<StableQuickSort<RotationPartition, AuxiliaryRotation>>("rotation, auxiliary");
        sorts_stably::<StableQuickSort<SlidingPartition, AuxiliaryRotation>>("sliding, auxiliary");
    }

    fn moves_are_all_logged<S: DirectSort>(name: &str) {
        let mut rng = Rng(0x10661);
        for n in [32usize, 33, 100, 257, 600, 1500] {
            for values in [1, 2, 5, 12, 40, 100_000] {
                let mut arr: Vec<usize> = (0..n).map(|_| rng.next(values)).collect();
                let mut replay = Replay::of(&arr);
                S::sort(&mut arr, &mut replay);
                assert!(arr.windows(2).all(|w| w[0] <= w[1]), "{name}: not sorted, n={n} values={values}");
                assert_eq!(replay.main(), &arr[..], "{name}: a move went unlogged, n={n} values={values}");
            }
        }
    }

    // The replay's scratch starts blank and the sort's holds garbage, so a read
    // of an unwritten cell would leave the two apart.
    #[test]
    fn no_shape_moves_an_element_without_logging_it() {
        moves_are_all_logged::<StableQuickSort<RotationPartition, ReversalRotation>>("rotation, reversal");
        moves_are_all_logged::<StableQuickSort<EncodedPartition, ReversalRotation>>("encoded, reversal");
        moves_are_all_logged::<StableQuickSort<EncodedPartition, TrinityRotation>>("encoded, trinity");
        moves_are_all_logged::<StableQuickSort<EncodedPartition, AuxiliaryRotation>>("encoded, auxiliary");
    }

    // One aux array for the run, whichever of the partition and the rotation
    // asks for more.
    #[test]
    fn the_scratch_is_one_aux_array_of_the_run() {
        use sort_logger::{SortLog, VisualizerLogger};
        let mut arr: Vec<usize> = (0..5000).map(|i| i * 7919 % 5003).collect();
        let mut log = VisualizerLogger { type_ghost: core::marker::PhantomData, log: Vec::new() };
        StableQuickSort::<EncodedPartition, ReversalRotation>::sort(&mut arr, &mut log);
        let created = log.log.iter().filter(|e| matches!(e, SortLog::CreateAuxArrT { .. })).count();
        assert_eq!(created, 1);
    }

    // A comparison per element per level of a split that halves, give or take
    // the three way split's second pass over the upper half.
    #[test]
    fn costs_n_log_n_comparisons_on_random_keys() {
        let n = 1 << 16;
        let mut rng = Rng(0xC0FFEE);
        let mut arr: Vec<usize> = (0..n).map(|_| rng.next(8 * n)).collect();
        let mut tally = Tally::default();
        StableQuickSort::<EncodedPartition, ReversalRotation>::sort(&mut arr, &mut tally);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        assert!(tally.cmps < 3 * n * 16, "comparisons {}", tally.cmps);
        assert!(tally.moves < 8 * n * 16, "moves {}", tally.moves);
    }

    // A partition that splits as badly as can be, around the least element of the
    // region whatever pivot it is given: the stack still stays small, since the
    // sort loops on the larger side and recurses into the smaller, here empty,
    // one. The stack is read off the address of a local in each call.
    struct Skewed;
    thread_local! {
        static LOWEST: core::cell::Cell<usize> = const { core::cell::Cell::new(usize::MAX) };
    }
    impl StablePartition for Skewed {
        const NAME: &'static str = "skewed";
        const MOVES: Complexity = Complexity::N_LOG_N;

        fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            _pivot: T,
            scratch: &mut [T],
            log: &mut U,
        ) -> usize {
            let here = 0u8;
            LOWEST.with(|c| c.set(c.get().min(&here as *const u8 as usize)));
            let least = *arr[lo..hi].iter().min().unwrap();
            RotationPartition::partition::<R, T, U>(arr, lo, hi, least, scratch, log)
        }

        fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            _pivot: T,
            scratch: &mut [T],
            log: &mut U,
        ) -> usize {
            let least = *arr[lo..hi].iter().min().unwrap();
            RotationPartition::partition_at_most::<R, T, U>(arr, lo, hi, least, scratch, log)
        }
    }

    #[test]
    fn the_stack_stays_small_when_every_split_is_lopsided() {
        let n = 6000;
        let mut arr = input(0, n, &mut Rng(3));
        let top = 0u8;
        let start = &top as *const u8 as usize;
        LOWEST.with(|c| c.set(usize::MAX));
        StableQuickSort::<Skewed, ReversalRotation>::sort(&mut arr, &mut NoOpLogger);
        assert!(arr.windows(2).all(|w| w[0].key <= w[1].key));
        let used = start - LOWEST.with(|c| c.get()).min(start);
        assert!(used < 64 * 1024, "the sort used {used} bytes of stack for {n} elements");
    }

    #[test]
    fn the_bounds_follow_from_the_axes() {
        type Encoded = StableQuickSort<EncodedPartition, ReversalRotation>;
        type Rotating = StableQuickSort<RotationPartition, ReversalRotation>;
        assert_eq!(<Encoded as HasTimeBounds>::WORST, Complexity::N_SQUARED);
        assert_eq!(<Encoded as HasTimeBounds>::AVERAGE, Complexity::N_LOG_N);
        assert_eq!(<Rotating as HasTimeBounds>::WORST, Complexity::N_SQUARED_LOG_N);
        assert_eq!(<Rotating as HasTimeBounds>::AVERAGE, Complexity::N_LOG_SQUARED);
        assert_eq!(<Encoded as HasSpace>::SPACE, Complexity::LOG_N);
        assert_eq!(<StableQuickSort<EncodedPartition, AuxiliaryRotation> as HasSpace>::SPACE, Complexity::N1);
        assert!(<Encoded as HasStability>::STABLE);
    }
}
