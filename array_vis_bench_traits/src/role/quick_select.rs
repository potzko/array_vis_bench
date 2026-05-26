//! `QuickSelect` — partition-based k-th-order-statistic finder role.
//!
//! Implementors reorder `arr` so the element that would sit at index
//! `target` after a full sort ends up there. The two surrounding
//! partitions remain unordered.

use sort_logger::SortLogger;

pub trait QuickSelect {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
        target: usize,
    );
}
