use std::marker::PhantomData;

use crate::traits::log_traits::SortLogger;

use super::sequences::GapSequence;

/// Generic shell sort parameterised on a gap sequence strategy.
///
/// This type is NOT registered directly — concrete registration wrappers
/// (one per sequence type) are generated in `sequences.rs`.  Those wrappers
/// delegate their `sort` call here so the algorithm lives in exactly one place.
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
