//! Top-level deep-heapify strategy — how to turn the whole array into a
//! heap before extraction starts.
//!
//! Two strategies:
//!
//! - [`Recursive`] — depth-first: each child subtree is heapified first,
//!   then the current node. This is just [`Heap::deep_heapify`] from the
//!   root.
//! - [`Iterative`] — the textbook bottom-up loop: heapify each internal
//!   node, scanning from the last internal node down to the root.
//!
//! Both are O(n), but produce visibly different traces.

use super::arity::Arity;
use super::heap::Heap;
use crate::traits::log_traits::SortLogger;

pub trait DeepHeapify {
    fn deep_heapify<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    );
}

pub struct Recursive;
combo_codegen::component!(DeepHeapify, Recursive, "recursive");

impl DeepHeapify for Recursive {
    #[inline(always)]
    fn deep_heapify<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        H::deep_heapify(arr, arr.len(), 0, logger);
    }
}

pub struct Iterative;
combo_codegen::component!(DeepHeapify, Iterative, "iterative");

impl DeepHeapify for Iterative {
    #[inline(always)]
    fn deep_heapify<H: Heap, T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        logger: &mut U,
    ) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        // Last internal node = parent of the last leaf. For an A-ary heap
        // children of `i` live at `A*i+1 ..= A*i+A`, so the parent of node
        // `k` is `(k - 1) / A` and the parent of the final leaf `n - 1` is
        // `(n - 2) / A`.
        let arity = <H::Arity as Arity>::N;
        let last_internal = (n - 2) / arity;
        for i in (0..=last_internal).rev() {
            H::heapify(arr, n, i, logger);
        }
    }
}
