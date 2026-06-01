use array_vis_bench_traits::role::pivot::median_index;
use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PivotQuality, PivotSelector,
};
use sort_logger::SortLogger;

pub struct MedianOfThree;

impl PivotSelector for MedianOfThree {
    const NAME: &'static str = "median of 3";
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> usize {
        median_index(arr, logger, 0, arr.len() / 2, arr.len() - 1)
    }
}

impl HasTimeBounds for MedianOfThree {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for MedianOfThree {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for MedianOfThree {
    const STABLE: bool = true;
}
impl PivotQuality for MedianOfThree {
    // Reduces the probability of a bad pivot but doesn't eliminate
    // worst-case quadratic behaviour.
    const DEGENERATES: bool = true;
}
