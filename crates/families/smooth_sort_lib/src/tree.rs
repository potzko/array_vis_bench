//! Moves and comparisons on the logical positions of a heap: `above` asks
//! whether one cell outranks another, `swap`, `put` and `copy` write one, and
//! `children` finds the two subtrees of a tree from its root. A position is
//! logical, the direction `D` maps it to a cell of the array.

use heap_sort_lib::compare::Compare;
use heap_sort_lib::direction::Direction;
use heap_sort_lib::layout::Layout;
use sort_logger::SortLogger;

use crate::shape::Shape;

#[inline(always)]
fn cell<D: Direction>(n: usize, at: usize) -> usize { <D::Layout as Layout>::phys(at, n) }

/// Whether the element at `a` is strictly more rootward than the one at `b`:
/// the greater one under a max direction, the smaller under a min.
#[inline(always)]
pub fn above<D: Direction, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], a: usize, b: usize, log: &mut U) -> bool {
    let n = arr.len();
    <D::Compare as Compare>::comes_first(log, arr, cell::<D>(n, a), cell::<D>(n, b))
}

#[inline(always)]
pub fn swap<D: Direction, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], a: usize, b: usize, log: &mut U) {
    let n = arr.len();
    log.swap(arr, cell::<D>(n, a), cell::<D>(n, b));
}

/// The element at `at`, held while its cell is written over.
#[inline(always)]
pub fn take<D: Direction, T: Ord + Copy>(arr: &[T], at: usize) -> T { arr[cell::<D>(arr.len(), at)] }

#[inline(always)]
pub fn put<D: Direction, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], at: usize, value: T, log: &mut U) {
    let n = arr.len();
    log.write_data(arr, cell::<D>(n, at), value);
}

/// The root and the order of the left and of the right subtree of the tree of
/// `order` rooted at `root`, or none for one element. The right subtree ends
/// just before the root and the left one just before the right.
#[inline(always)]
pub fn children<S: Shape>(root: usize, order: usize) -> Option<[(usize, usize); 2]> {
    S::children(order).map(|(left, right)| {
        let right_root = root - 1;
        [(right_root - S::size(right), left), (right_root, right)]
    })
}
