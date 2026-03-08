use std::marker::PhantomData;

use crate::traits::log_traits::SortLogger;

use super::sequences::GapSequence;

/// Shell sort variant that completes each gap-subsequence before advancing.
///
/// Standard shell sort sweeps left-to-right for each gap, partially sorting
/// each subsequence as it goes.  This variant instead finishes one full
/// subsequence (start, start+gap, start+2*gap, ...) via insertion sort before
/// moving to the next start offset — the same asymptotic behaviour but a
/// visually distinct access pattern.
pub struct ShellSortOrdered<Seq: GapSequence> {
    _phantom: PhantomData<Seq>,
}

impl<Seq: GapSequence> ShellSortOrdered<Seq> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let gaps = Seq::gaps(arr.len());
        for &gap in &gaps {
            for start in 0..gap {
                let mut i = start + gap;
                while i < arr.len() {
                    let mut ii = i;
                    while ii >= gap + start {
                        if !logger.cond_swap_lt(arr, ii, ii - gap) {
                            break;
                        }
                        ii -= gap;
                    }
                    i += gap;
                }
            }
        }
    }
}
