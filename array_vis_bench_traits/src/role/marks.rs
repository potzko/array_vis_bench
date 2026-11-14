//! `Marks`, a flag per cell of a buffer: the dead leaves of a level heap, a
//! cell marked when the hole dies in it. The axis is where the flags are kept.
//! A filler allocates a `Store` outside the array, or carves `CARVED` elements
//! of the array per mark and keeps the flag in their order, or both are empty
//! and it keeps nothing. Every mark starts clear, `mark` is only asked of a
//! clear one and `unmark` of a marked one, so neither has to read first.

use sort_logger::SortLogger;

pub trait Marks {
    /// Short label used in the generated variant name.
    const NAME: &'static str;

    /// Bits allocated outside the array per mark, `0` when nothing is.
    const ALLOCATED_BITS: usize;

    /// Elements of the array a mark is carved from, `0` when it is allocated.
    /// The carved region is `CARVED` distinct elements per mark, sorted, and
    /// is left sorted once every mark is clear again.
    const CARVED: usize;

    /// What is allocated for the marks, `()` when they are carved.
    type Store;

    /// A store of `cells` clear marks, allocated through the logger.
    fn create<T: Ord + Copy, U: ?Sized + SortLogger<T>>(cells: usize, log: &mut U) -> Self::Store;

    fn free<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &Self::Store, log: &mut U);

    /// Whether the mark `i` is set. `carved` is the region the marks were
    /// carved from, empty for a filler that allocates.
    fn is_marked<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &Self::Store, carved: &[T], i: usize, log: &mut U) -> bool;

    /// Set the mark `i`, which is clear.
    fn mark<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &mut Self::Store, carved: &mut [T], i: usize, log: &mut U);

    /// Clear the mark `i`, which is set.
    fn unmark<T: Ord + Copy, U: ?Sized + SortLogger<T>>(store: &mut Self::Store, carved: &mut [T], i: usize, log: &mut U);
}
