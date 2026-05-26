use array_vis_bench_traits::role::pivot::median_index;
use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PivotQuality, PivotSelector,
};
use sort_logger::SortLogger;

pub struct Ninther;

impl PivotSelector for Ninther {
    const NAME: &'static str = "ninther";
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &[T],
        logger: &mut U,
    ) -> usize {
        let len = arr.len();
        if len < 9 {
            return median_index(arr, logger, 0, len / 2, len - 1);
        }
        // 9 evenly spaced samples, grouped into 3 triples.
        let s = [
            0, len / 8, len / 4,              // Group A
            3 * len / 8, len / 2, 5 * len / 8, // Group B
            3 * len / 4, 7 * len / 8, len - 1, // Group C
        ];
        let m1 = median_index(arr, logger, s[0], s[1], s[2]);
        let m2 = median_index(arr, logger, s[3], s[4], s[5]);
        let m3 = median_index(arr, logger, s[6], s[7], s[8]);
        median_index(arr, logger, m1, m2, m3)
    }
}

impl HasTimeBounds for Ninther {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for Ninther {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for Ninther {
    const STABLE: bool = true;
}
impl PivotQuality for Ninther {
    const DEGENERATES: bool = true;
}
