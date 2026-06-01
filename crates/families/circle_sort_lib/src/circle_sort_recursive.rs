use sort_logger::SortLogger;
use super::orderings::RecursiveOrder;

/// Generic recursive circle sort, parameterised over the ordering of its
/// three operations at each recursion level.
///
/// See [`RecursiveOrder`] for the abstraction and
/// `orderings.rs` for the concrete variants.
pub struct CircleSortRecursive<Order>(std::marker::PhantomData<Order>);

impl<Order: RecursiveOrder> CircleSortRecursive<Order> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        while Order::sort_range(arr, 0, arr.len() - 1, logger) {}
    }
}
