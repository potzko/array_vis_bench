use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme,
};
use sort_logger::SortLogger;

/// Three-way partition (Dutch National Flag).
///
/// Splits into three regions: `< pivot`, `== pivot`, `> pivot`.
/// Equal elements are grouped in the middle and excluded from recursion.
pub struct ThreeWay;

impl PartitionScheme for ThreeWay {
    const NAME: &'static str = "three-way";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        logger.swap(arr, pivot_idx, 0);
        let pivot = arr[0];

        let mut lt = 0; // end of "< pivot" region
        let mut i = 1; // scan pointer
        let mut gt = arr.len() - 1; // start of "> pivot" region

        while i <= gt {
            if logger.cmp_lt_data(arr, i, pivot) {
                logger.swap(arr, i, lt);
                lt += 1;
                i += 1;
            } else if logger.cmp_gt_data(arr, i, pivot) {
                logger.swap(arr, i, gt);
                if gt == 0 {
                    break;
                }
                gt -= 1;
                // don't advance i — swapped-in element not yet examined
            } else {
                i += 1; // == pivot
            }
        }
        (lt, gt + 1)
    }
}

impl HasTimeBounds for ThreeWay {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for ThreeWay {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for ThreeWay {
    const STABLE: bool = false;
}
