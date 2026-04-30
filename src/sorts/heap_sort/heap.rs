//! Heap data structure abstraction.
//!
//! Implementors expose three primitive operations heap-using sorts need:
//! [`Heap::heapify`] (single sift-down), [`Heap::deep_heapify`] (recursive
//! subtree build), and [`Heap::swap`] (swap two logical heap positions —
//! lets the caller stay layout-agnostic). The arity is encoded in the impl
//! via the associated `Arity` type.

use super::arity::Arity;
use crate::traits::log_traits::SortLogger;

pub trait Heap {
    type Arity: Arity;

    /// Single-node sift-down. Assume each child's subtree is already a heap;
    /// move logical node `i` toward the leaves until the heap invariant
    /// holds at `i`.
    fn heapify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        heap_size: usize,
        i: usize,
        logger: &mut U,
    );

    /// Recursively make the entire subtree rooted at `i` a heap.
    fn deep_heapify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        heap_size: usize,
        i: usize,
        logger: &mut U,
    );

    /// Swap two logical heap positions. Each impl translates logical to
    /// physical via its layout (identity for forward, `n - 1 - i` for
    /// reverse), so callers don't have to know the layout.
    fn swap<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        i: usize,
        j: usize,
        logger: &mut U,
    );
}
