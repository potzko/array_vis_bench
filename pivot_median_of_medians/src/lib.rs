use array_vis_bench_traits::role::pivot::median_index;
use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PivotQuality, PivotSelector,
};
use sort_logger::SortLogger;

pub struct MedianOfMedians;

impl PivotSelector for MedianOfMedians {
    const NAME: &'static str = "median of medians";
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> usize {
        let len = arr.len();
        if len < 5 {
            return len / 2;
        }
        let samples = [0, len / 4, len / 2, (3 * len) / 4, len - 1];
        let m1 = median_index(arr, logger, samples[0], samples[1], samples[2]);
        let m2 = median_index(arr, logger, samples[2], samples[3], samples[4]);
        median_index(arr, logger, m1, samples[2], m2)
    }
}

// True linear-time pivot — guarantees a balanced split so QuickSort's
// worst case becomes O(N log N).
impl HasTimeBounds for MedianOfMedians {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for MedianOfMedians {
    // Recursive selection on groups of 5 — depth is O(log N) stack.
    const SPACE: Complexity = Complexity::LOG_N;
}
impl HasStability for MedianOfMedians {
    const STABLE: bool = true;
}
impl PivotQuality for MedianOfMedians {
    const DEGENERATES: bool = false;
}
