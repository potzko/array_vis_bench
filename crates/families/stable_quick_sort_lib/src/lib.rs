//! Stable quick sort, `StableQuickSort<P, R>`: a quick sort whose split is a
//! stable partition `P`, rotating with `R`, so that equal elements never cross
//! each other and the sort needs no tag and no index. `sort` is the driver,
//! `pivot_of` reads the pivot value off a median of three, a ninther past
//! `NINTHER` elements, and `sort_range` splits, with a second pass for the equals
//! of the pivot only when the first left nothing below it, recurses into the
//! smaller side and loops on the larger. The family is declared in
//! `stable_quick.spec` and built by `stable_quick.avbs`.

use core::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::role::keys::sort_region;
use array_vis_bench_traits::role::pivot::median_index;
use array_vis_bench_traits::role::{with_words, Rotation, StablePartition};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

#[cfg(test)]
pub mod test_util;

/// `P` is the `StablePartition` that splits a region around the pivot and `R` the
/// `Rotation` it rotates with. The scratch the two share is one allocation of the
/// sort, and the fixed size words of the partition are the other.
pub struct StableQuickSort<P, R>(PhantomData<(P, R)>);

impl<P: StablePartition, R: Rotation> StableQuickSort<P, R> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], log: &mut U) {
        let n = arr.len();
        if n < 2 { return; }
        let cells = R::scratch_size(n).max(P::scratch_size(n));
        let mut scratch = if cells > 0 { log.create_aux_arr_t(cells) } else { Vec::new() };
        with_words::<T, U, _>(log, P::SCRATCH_SECTIONS, |log, words| sort_range::<P, R, T, U>(arr, 0, n, &mut scratch, words, log));
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

// Split `arr[lo..hi]` around the pivot into below and the rest, each in order,
// sort the below and the rest, and loop. When nothing is below the pivot the rest
// is the whole region and the split has made no progress, so a second pass,
// over the region again, peels the equals of the pivot off, which are in place
// and end the region when they are all of it; a region of distinct keys never
// needs the pass, and the first pass alone is cheaper than both passes by about a
// quarter of the moves and the comparisons. The smaller of the two sides is the
// recursion and the larger the next turn of the loop, so the stack is `log n`
// frames deep whatever the pivots do.
fn sort_range<P: StablePartition, R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    lo: usize,
    hi: usize,
    scratch: &mut [T],
    words: &mut [usize],
    log: &mut U,
) {
    let (mut lo, mut hi) = (lo, hi);
    while hi - lo > SMALL {
        let pivot = pivot_of(arr, lo, hi, log);
        let equal = P::partition::<R, T, U>(arr, lo, hi, pivot, scratch, words, log);
        let above = if equal == lo { P::partition_at_most::<R, T, U>(arr, lo, hi, pivot, scratch, words, log) } else { equal };
        if equal - lo < hi - above {
            sort_range::<P, R, T, U>(arr, lo, equal, scratch, words, log);
            lo = above;
        } else {
            sort_range::<P, R, T, U>(arr, above, hi, scratch, words, log);
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
    use extraction_lib::{EncodedPartition, MaskedPartition, MaskedTablePartition, RotationPartition, SlidingPartition};
    use rotation_auxiliary::AuxiliaryRotation;
    use rotation_reversal::ReversalRotation;
    use rotation_trinity::TrinityRotation;
    use sort_logger::{split_sections, NoOpLogger};

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
        sorts_stably::<StableQuickSort<MaskedPartition, ReversalRotation>>("masked, reversal");
        sorts_stably::<StableQuickSort<MaskedTablePartition, ReversalRotation>>("masked table, reversal");
    }

    // The rotation is `R`'s, and trinity and the auxiliary rotation run through
    // the scratch the sort allocates once.
    #[test]
    fn any_rotation_sorts_stably() {
        sorts_stably::<StableQuickSort<EncodedPartition, TrinityRotation>>("encoded, trinity");
        sorts_stably::<StableQuickSort<EncodedPartition, AuxiliaryRotation>>("encoded, auxiliary");
        sorts_stably::<StableQuickSort<MaskedPartition, TrinityRotation>>("masked, trinity");
        sorts_stably::<StableQuickSort<MaskedPartition, AuxiliaryRotation>>("masked, auxiliary");
        sorts_stably::<StableQuickSort<MaskedTablePartition, AuxiliaryRotation>>("masked table, auxiliary");
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
        moves_are_all_logged::<StableQuickSort<MaskedPartition, ReversalRotation>>("masked, reversal");
        moves_are_all_logged::<StableQuickSort<MaskedPartition, TrinityRotation>>("masked, trinity");
        moves_are_all_logged::<StableQuickSort<MaskedPartition, AuxiliaryRotation>>("masked, auxiliary");
        moves_are_all_logged::<StableQuickSort<MaskedTablePartition, ReversalRotation>>("masked table, reversal");
        moves_are_all_logged::<StableQuickSort<MaskedTablePartition, AuxiliaryRotation>>("masked table, auxiliary");
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

    // The masked partitions ask for no scratch of elements, so over a rotation that
    // asks for none the sort allocates no array of them.
    #[test]
    fn the_masked_partitions_over_a_rotation_without_scratch_allocate_nothing() {
        use sort_logger::{SortLog, VisualizerLogger};
        let mut arr: Vec<usize> = (0..5000).map(|i| i * 7919 % 5003).collect();
        let mut log = VisualizerLogger { type_ghost: core::marker::PhantomData, log: Vec::new() };
        StableQuickSort::<MaskedPartition, ReversalRotation>::sort(&mut arr, &mut log);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(log.log.iter().filter(|e| matches!(e, SortLog::CreateAuxArrT { .. })).count(), 0);
        let mut arr: Vec<usize> = (0..5000).map(|i| i * 7919 % 5003).collect();
        let mut log = VisualizerLogger { type_ghost: core::marker::PhantomData, log: Vec::new() };
        StableQuickSort::<MaskedTablePartition, ReversalRotation>::sort(&mut arr, &mut log);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(log.log.iter().filter(|e| matches!(e, SortLog::CreateAuxArrT { .. })).count(), 0);
    }

    // A partition by rotation that keeps words of its own, three sections with an
    // empty one between, whose length it checks on every call and which it writes
    // to through the logger, the last section's last word.
    struct Wordy;
    impl StablePartition for Wordy {
        const NAME: &'static str = "wordy";
        const MOVES: Complexity = Complexity::N_LOG_N;
        const SCRATCH_SECTIONS: &'static [usize] = &[3, 0, 5];

        fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            pivot: T,
            scratch: &mut [T],
            words: &mut [usize],
            log: &mut U,
        ) -> usize {
            assert_eq!(words.len(), 8, "the words of a call");
            let [_, _, tail] = split_sections(&mut *words, [3, 0, 5]);
            log.write_data_u(tail, 4, hi - lo);
            RotationPartition::partition::<R, T, U>(arr, lo, hi, pivot, scratch, words, log)
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
            assert_eq!(words.len(), 8, "the words of a call");
            let [_, _, tail] = split_sections(&mut *words, [3, 0, 5]);
            log.write_data_u(tail, 4, hi - lo);
            RotationPartition::partition_at_most::<R, T, U>(arr, lo, hi, pivot, scratch, words, log)
        }
    }

    // The names of the arrays of words a sort created, those it freed, and the
    // writes to words. The arrays of elements are none of the three.
    fn word_events(log: &sort_logger::VisualizerLogger<usize>) -> (Vec<usize>, Vec<usize>, usize) {
        use sort_logger::SortLog;
        let created: Vec<usize> = log.log.iter().filter_map(|e| if let SortLog::CreateAuxArr { name, .. } = e { Some(*name) } else { None }).collect();
        let freed = log.log.iter().filter_map(|e| if let SortLog::FreeAuxArr { name } = e { Some(*name) } else { None });
        let freed = freed.filter(|name| created.contains(name)).collect();
        (created, freed, log.log.iter().filter(|e| matches!(e, SortLog::WriteDataU { .. })).count())
    }

    // The words are one allocation of the sort, an array for each section that has
    // cells, however many partitions use them.
    #[test]
    fn the_words_are_created_once_and_freed_once_per_sort() {
        use sort_logger::VisualizerLogger;
        let mut arr: Vec<usize> = (0..5000).map(|i| i * 7919 % 5003).collect();
        let mut log = VisualizerLogger { type_ghost: PhantomData, log: Vec::new() };
        StableQuickSort::<Wordy, ReversalRotation>::sort(&mut arr, &mut log);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        let (created, freed, writes) = word_events(&log);
        assert_eq!((created.len(), &freed), (2, &created));
        assert!(writes > 100, "{writes} writes to the words: a partition each");
    }

    #[test]
    fn a_partition_without_words_makes_no_arrays_of_them() {
        use sort_logger::VisualizerLogger;
        for sort in [StableQuickSort::<RotationPartition, ReversalRotation>::sort, StableQuickSort::<EncodedPartition, ReversalRotation>::sort] {
            let mut arr: Vec<usize> = (0..5000).map(|i| i * 7919 % 5003).collect();
            let mut log = VisualizerLogger { type_ghost: PhantomData, log: Vec::new() };
            sort(&mut arr, &mut log);
            assert_eq!(word_events(&log), (vec![], vec![], 0));
        }
    }

    // The lengths of the arrays of words, in the order they were created, that a
    // sort of 5000 elements by `S` leaves, those freed in the same order, and the
    // writes to them.
    fn words_of_a_sort<S: DirectSort>() -> (Vec<usize>, bool, usize) {
        use sort_logger::{SortLog, VisualizerLogger};
        let mut arr: Vec<usize> = (0..5000).map(|i| i * 7919 % 5003).collect();
        let mut log = VisualizerLogger { type_ghost: PhantomData, log: Vec::new() };
        S::sort(&mut arr, &mut log);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        let (created, freed, writes) = word_events(&log);
        let lengths = log.log.iter().filter_map(|e| if let SortLog::CreateAuxArr { length, .. } = e { Some(*length) } else { None });
        (lengths.collect(), freed == created, writes)
    }

    // The masked partitions keep their masks, counts and the walk's array in words:
    // three arrays side by side, made once for the whole sort and freed once, the
    // windows of every partition writing to them.
    #[test]
    fn the_masked_partitions_create_their_words_once_per_sort() {
        let (lengths, freed, writes) = words_of_a_sort::<StableQuickSort<MaskedPartition, ReversalRotation>>();
        assert_eq!((lengths, freed), (vec![16, 17, 16], true));
        assert!(writes > 1000, "{writes} writes to the words");
        let (lengths, freed, writes) = words_of_a_sort::<StableQuickSort<MaskedTablePartition, ReversalRotation>>();
        assert_eq!((lengths, freed), (vec![16, 17, 1024], true));
        assert!(writes > 1000, "{writes} writes to the words");
    }

    // A comparison per element per level of a split that halves, give or take
    // the pivot's median of three and the insertion sorted leaves.
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
            words: &mut [usize],
            log: &mut U,
        ) -> usize {
            let here = 0u8;
            LOWEST.with(|c| c.set(c.get().min(&here as *const u8 as usize)));
            let least = *arr[lo..hi].iter().min().unwrap();
            RotationPartition::partition::<R, T, U>(arr, lo, hi, least, scratch, words, log)
        }

        fn partition_at_most<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            _pivot: T,
            scratch: &mut [T],
            words: &mut [usize],
            log: &mut U,
        ) -> usize {
            let least = *arr[lo..hi].iter().min().unwrap();
            RotationPartition::partition_at_most::<R, T, U>(arr, lo, hi, least, scratch, words, log)
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

    // A partition by rotation that counts its calls of the second kind, the pass
    // that peels the equals of the pivot off.
    struct Spy;
    thread_local! {
        static AT_MOST: core::cell::Cell<usize> = const { core::cell::Cell::new(0) };
    }
    impl StablePartition for Spy {
        const NAME: &'static str = "spy";
        const MOVES: Complexity = Complexity::N_LOG_N;

        fn partition<R: Rotation, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
            arr: &mut [T],
            lo: usize,
            hi: usize,
            pivot: T,
            scratch: &mut [T],
            words: &mut [usize],
            log: &mut U,
        ) -> usize {
            RotationPartition::partition::<R, T, U>(arr, lo, hi, pivot, scratch, words, log)
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
            AT_MOST.with(|c| c.set(c.get() + 1));
            RotationPartition::partition_at_most::<R, T, U>(arr, lo, hi, pivot, scratch, words, log)
        }
    }

    fn at_most_passes(arr0: &[Item]) -> usize {
        let mut arr = arr0.to_vec();
        AT_MOST.with(|c| c.set(0));
        StableQuickSort::<Spy, ReversalRotation>::sort(&mut arr, &mut NoOpLogger);
        let mut want = arr0.to_vec();
        want.sort_by_key(|x| x.key);
        assert_eq!(pairs(&arr), pairs(&want));
        AT_MOST.with(|c| c.get())
    }

    // The pass that peels the equals off is for a split that left nothing below
    // the pivot, since the rest would be the whole region again: a region of
    // distinct keys never needs it, a region of one value needs it once.
    #[test]
    fn the_second_pass_is_made_only_when_nothing_is_below_the_pivot() {
        let mut rng = Rng(0xD15);
        assert_eq!(at_most_passes(&items(&(0..3000).map(|_| rng.next(1 << 30) as u32).collect::<Vec<_>>())), 0);
        assert_eq!(at_most_passes(&items(&(0..3000).rev().collect::<Vec<u32>>())), 0);
        assert_eq!(at_most_passes(&items(&vec![7; 3000])), 1);
    }

    // The regions that need the second pass end and stay stable: every key
    // equal, a pivot that is the least key of its region, in every level, and
    // few values with a great many of each.
    #[test]
    fn a_region_of_equal_keys_and_one_with_its_minimum_for_a_pivot_end_and_stay_stable() {
        fn sorts<S: DirectSort>(name: &str) {
            let mut rng = Rng(0xE0);
            for n in [17usize, 40, 129, 1000, 6000] {
                let minimum_pivots: Vec<u32> = (0..n).map(|i| if i % 3 == 0 || i == n / 2 || i == n - 1 { 0 } else { 1 + rng.next(n) as u32 }).collect();
                let shapes = [vec![9u32; n], minimum_pivots, (0..n).map(|i| (i % 2) as u32).collect(), (0..n).map(|i| u32::from(i > n / 2)).collect()];
                for (shape, keys) in shapes.iter().enumerate() {
                    let arr0 = items(keys);
                    let mut arr = arr0.clone();
                    S::sort(&mut arr, &mut NoOpLogger);
                    let mut want = arr0.clone();
                    want.sort_by_key(|x| x.key);
                    assert_eq!(pairs(&arr), pairs(&want), "{name}: shape {shape}, n={n}");
                }
            }
        }
        sorts::<StableQuickSort<RotationPartition, ReversalRotation>>("rotation");
        sorts::<StableQuickSort<EncodedPartition, ReversalRotation>>("encoded");
        sorts::<StableQuickSort<EncodedPartition, AuxiliaryRotation>>("encoded, auxiliary");
        sorts::<StableQuickSort<MaskedPartition, ReversalRotation>>("masked");
        sorts::<StableQuickSort<MaskedTablePartition, ReversalRotation>>("masked table");
        sorts::<StableQuickSort<SlidingPartition, ReversalRotation>>("sliding");
    }

    // One pass of the partition a level, and a second only when needed: on
    // distinct keys the comparisons and moves are those of the first alone.
    #[test]
    fn distinct_keys_cost_one_pass_of_the_partition_a_level() {
        let n = 1 << 16;
        let mut rng = Rng(0xBEE);
        let mut arr: Vec<usize> = (0..n).map(|_| rng.next(8 * n)).collect();
        let mut tally = Tally::default();
        StableQuickSort::<EncodedPartition, ReversalRotation>::sort(&mut arr, &mut tally);
        assert!(arr.windows(2).all(|w| w[0] <= w[1]));
        assert!(tally.cmps < 21 * n, "comparisons {} an element", tally.cmps as f64 / n as f64);
        assert!(tally.moves < 45 * n, "moves {} an element", tally.moves as f64 / n as f64);
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
        assert_eq!(<StableQuickSort<MaskedPartition, ReversalRotation> as HasSpace>::SPACE, Complexity::LOG_N);
        assert_eq!(<StableQuickSort<MaskedPartition, ReversalRotation> as HasTimeBounds>::AVERAGE, Complexity::N_LOG_N);
        assert_eq!(<StableQuickSort<EncodedPartition, AuxiliaryRotation> as HasSpace>::SPACE, Complexity::N1);
        assert!(<Encoded as HasStability>::STABLE);
    }
}
