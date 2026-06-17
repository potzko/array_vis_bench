use std::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::{Complexity, NonTrivialSmallSort};
use sort_logger::SortLogger;

pub struct OddEvenBubbleSort<S: NonTrivialSmallSort> {
    _phantom: PhantomData<S>,
}

// Composable annotations (spec compiler inherits these). Adaptive via the
// `mutated` early-exit flag (O(N) best on sorted input); Θ(N²) average/worst;
// in-place; stable.
impl<S: NonTrivialSmallSort> HasTimeBounds for OddEvenBubbleSort<S> {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}
impl<S: NonTrivialSmallSort> HasSpace for OddEvenBubbleSort<S> {
    const SPACE: Complexity = Complexity::CONST;
}
impl<S: NonTrivialSmallSort> HasStability for OddEvenBubbleSort<S> {
    const STABLE: bool = true;
}

impl<S: NonTrivialSmallSort> OddEvenBubbleSort<S> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() <= S::THRESHOLD {
            S::sort(arr, logger);
            return;
        }
        let mut mutated = true;
        while mutated {
            mutated = false;
            // Even pass: blocks starting at 0, T, 2T, …. The last block is
            // clamped to arr.len() so a tail shorter than T still gets sorted.
            for start in (0..arr.len()).step_by(S::THRESHOLD) {
                let end = (start + S::THRESHOLD).min(arr.len());
                mutated |= S::sort(&mut arr[start..end], logger);
            }
            // Odd pass: blocks offset by T/2 — overlaps the even-pass blocks
            // at every boundary so adjacent pairs converge.
            for start in ((S::THRESHOLD / 2)..arr.len()).step_by(S::THRESHOLD) {
                let end = (start + S::THRESHOLD).min(arr.len());
                mutated |= S::sort(&mut arr[start..end], logger);
            }
        }
    }
}
