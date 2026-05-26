use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PartitionScheme, PartitionSchemeV,
    PartitionVisitor,
};
use sort_logger::SortLogger;

/// Moving-pivot partition.
///
/// Swaps the selected pivot to the start, then walks inward: elements
/// smaller than the current head extend the low region, larger elements
/// are swapped to the high end.
pub struct MovingPivot;

impl PartitionScheme for MovingPivot {
    const NAME: &'static str = "moving pivot";
    fn partition<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        pivot_idx: usize,
    ) -> (usize, usize) {
        logger.swap(arr, pivot_idx, 0);

        let mut low = 0;
        let mut high = arr.len() - 1;
        while low < high - 1 {
            if logger.cond_swap_le(arr, low + 1, low) {
                low += 1;
            } else {
                logger.swap(arr, low + 1, high);
                high -= 1;
            }
        }
        logger.cond_swap_lt(arr, high, low);
        (high, high)
    }
}

impl PartitionSchemeV for MovingPivot {
    const NAME: &'static str = "moving pivot";
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

        let mut low = 0;
        let mut high = len - 1;
        while low < high - 1 {
            if logger.cond_swap_le(arr, low + 1, low) {
                low += 1;
            } else {
                logger.swap(arr, low + 1, high);
                high -= 1;
            }
        }
        logger.cond_swap_lt(arr, high, low);
        // Mirrors the tuple impl: `(high, high)` — the pivot is reprocessed
        // in the right sub-call (it's at index `high` and `arr[high..]`
        // includes it). Same convention here for parity.
        visitor.unsorted(0..high);
        visitor.unsorted(high..len);
    }
}

impl HasTimeBounds for MovingPivot {
    const WORST: Complexity = Complexity::N1;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N1;
}
impl HasSpace for MovingPivot {
    const SPACE: Complexity = Complexity::CONST;
}
impl HasStability for MovingPivot {
    const STABLE: bool = false;
}
