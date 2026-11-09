use std::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

use crate::sequences::GapSequence;

/// Generic shell sort parameterised on a gap sequence strategy.
///
/// This type is NOT registered directly — concrete registration wrappers
/// (one per sequence type) are generated in `registration.rs`. Those
/// wrappers delegate their `sort` call here so the algorithm lives in
/// exactly one place.
pub struct ShellSort<Seq: GapSequence> {
    _phantom: PhantomData<Seq>,
}

impl<Seq: GapSequence> ShellSort<Seq> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let gaps = Seq::gaps(arr.len());
        for &gap in &gaps {
            for i in gap..arr.len() {
                insert_into_chain(arr, i, gap, logger);
            }
        }
    }
}

/// Inserts `arr[i]` into the sorted chain below it, `i - gap`, `i - 2 gap`,
/// and so on. Holds the element and shifts each larger predecessor up one gap,
/// so moving d places costs d + 1 writes where swapping cost 2d, for the same
/// comparisons.
pub fn insert_into_chain<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    i: usize,
    gap: usize,
    logger: &mut U,
) {
    let held = arr[i];
    let mut slot = i;
    while slot >= gap && !logger.cmp_le_data(arr, slot - gap, held) {
        logger.write(arr, slot, slot - gap);
        slot -= gap;
    }
    if slot != i { logger.write_data(arr, slot, held); }
}

array_vis_bench_traits::direct_sort!(ShellSort<Seq>; Seq: GapSequence);

// ── Composable annotations ──────────────────────────────────────────
//
// Shell sort's time complexity is determined entirely by the gap
// sequence — that's the whole point of the sequence parameter.
// Space is `O(1)` (in-place insertion at each gap). Shell sort is
// not stable for any gap sequence with `gap > 1`, so STABLE is
// always false regardless of Seq's own STABLE.

impl<Seq: GapSequence + HasTimeBounds> HasTimeBounds for ShellSort<Seq> {
    const WORST: Complexity = Seq::WORST;
    const BEST: Complexity = Seq::BEST;
    const AVERAGE: Complexity = Seq::AVERAGE;
}
impl<Seq: GapSequence> HasSpace for ShellSort<Seq> {
    const SPACE: Complexity = Complexity::CONST;
}
impl<Seq: GapSequence> HasStability for ShellSort<Seq> {
    const STABLE: bool = false;
}

#[cfg(test)]
mod annotation_tests {
    use super::*;
    use crate::sequences::{Ciura, ClassicGaps, Pratt};

    #[test]
    fn complexity_pulled_from_sequence() {
        assert_eq!(<ShellSort<ClassicGaps> as HasTimeBounds>::WORST, Complexity::N_SQUARED);
        assert_eq!(<ShellSort<Ciura> as HasTimeBounds>::WORST, Complexity::N_LOG_N);
        assert_eq!(<ShellSort<Pratt> as HasTimeBounds>::WORST, Complexity::N_LOG_SQUARED);
    }

    #[test]
    fn shell_sort_never_stable() {
        // Stability is a property of the algorithm, not the sequence.
        assert!(!<ShellSort<ClassicGaps> as HasStability>::STABLE);
        assert!(!<ShellSort<Pratt> as HasStability>::STABLE);
    }

    #[test]
    fn shell_sort_in_place() {
        assert!(<ShellSort<ClassicGaps> as HasSpace>::SPACE.is_in_place());
    }
}

#[cfg(test)]
mod kernel_tests {
    use std::marker::PhantomData;

    use super::*;
    use crate::ShellSortOrdered;
    use sort_logger::{SortLog, VisualizerLogger};

    // A single gap of 1, so either driver is a plain insertion sort.
    struct Ones;
    impl GapSequence for Ones {
        const NAME: &'static str = "ones";
        const BIG_O: &'static str = "O(N^2)";
        fn gaps(_: usize) -> Vec<usize> { vec![1] }
    }

    #[test]
    fn a_pass_writes_once_per_place_an_element_moves() {
        // A reversed run of five: the element at index i moves i places, i
        // shifts and one write of the held element, and nothing swaps.
        let sorts: [fn(&mut [u32], &mut VisualizerLogger<u32>); 2] = [ShellSort::<Ones>::sort, ShellSortOrdered::<Ones>::sort];
        for sort in sorts {
            let mut arr = [5, 4, 3, 2, 1];
            let mut logger = VisualizerLogger { type_ghost: PhantomData, log: Vec::new() };
            sort(&mut arr, &mut logger);
            assert_eq!(arr, [1, 2, 3, 4, 5]);
            let swaps = logger.log.iter().filter(|e| matches!(e, SortLog::Swap { .. })).count();
            let writes = logger.log.iter().filter(|e| matches!(e, SortLog::WriteInArr { .. } | SortLog::WriteData { .. })).count();
            assert_eq!((swaps, writes), (0, 10 + 4));
        }
    }
}
