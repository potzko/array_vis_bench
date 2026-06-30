use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

pub struct BubbleSort;

impl BubbleSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        for i in 0..arr.len() {
            for ii in 1..arr.len() - i {
                logger.cond_swap_lt(arr, ii, ii - 1);
            }
        }
    }
}

// Composable annotations (spec compiler inherits these). This implementation has
// no early-exit flag, so it is Θ(N²) on every input; in-place; stable.
impl HasTimeBounds for BubbleSort {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N_SQUARED;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}
impl HasSpace for BubbleSort {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for BubbleSort {
    const STABLE: bool = true;
}
