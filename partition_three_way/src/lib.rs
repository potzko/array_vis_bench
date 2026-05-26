use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionVisitor,
};
use sort_logger::SortLogger;

/// Three-way partition (Dutch National Flag).
///
/// Splits into three regions: `< pivot`, `== pivot`, `> pivot`.
/// Equal elements are grouped in the middle and excluded from recursion.
pub struct ThreeWay;

impl PartitionScheme for ThreeWay {
    const NAME: &'static str = "three-way";
    const N_PIVOTS: usize = 1;
    #[inline]
    fn partition<T, U, V>(
        arr: &mut [T],
        logger: &mut U,
        pivots: &[usize],
        visitor: &mut V,
    ) where
        T: Ord + Copy,
        U: ?Sized + SortLogger<T>,
        V: PartitionVisitor,
    {
        let len = arr.len();
        logger.swap(arr, pivots[0], 0);
        let pivot = arr[0];

        let mut lt = 0;
        let mut i = 1;
        let mut gt = len - 1;

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
            } else {
                i += 1;
            }
        }
        visitor.unsorted(0..lt);
        visitor.unsorted(gt + 1..len);
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
