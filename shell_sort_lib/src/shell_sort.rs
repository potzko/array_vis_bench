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
                let mut ii = i;
                while ii >= gap {
                    if !logger.cond_swap_lt(arr, ii, ii - gap) {
                        break;
                    }
                    ii -= gap;
                }
            }
        }
    }
}

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
    use crate::sequences::{Ciura, Classic, Pratt};

    #[test]
    fn complexity_pulled_from_sequence() {
        assert_eq!(<ShellSort<Classic> as HasTimeBounds>::WORST, Complexity::N_SQUARED);
        assert_eq!(<ShellSort<Ciura> as HasTimeBounds>::WORST, Complexity::N_LOG_N);
        assert_eq!(<ShellSort<Pratt> as HasTimeBounds>::WORST, Complexity::N_LOG_SQUARED);
    }

    #[test]
    fn shell_sort_never_stable() {
        // Stability is a property of the algorithm, not the sequence.
        assert!(!<ShellSort<Classic> as HasStability>::STABLE);
        assert!(!<ShellSort<Pratt> as HasStability>::STABLE);
    }

    #[test]
    fn shell_sort_in_place() {
        assert!(<ShellSort<Classic> as HasSpace>::SPACE.is_in_place());
    }
}
