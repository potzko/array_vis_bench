use sort_logger::SortLogger;
use super::directions::BottomUpDirection;

/// Generic bottom-up circle sort, parameterised over the size-traversal
/// direction.
///
/// See [`BottomUpDirection`] for the abstraction and `directions.rs` for the
/// concrete variants.
pub struct CircleSortBottomUp<Dir>(std::marker::PhantomData<Dir>);

impl<Dir: BottomUpDirection> CircleSortBottomUp<Dir> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        while Dir::run_iteration(arr, logger) {}
    }
}
