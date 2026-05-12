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
        // Each Heap impl knows its own "deepest internal node" formula —
        // n-ary heap uses `(n - 2) / A`, beap uses `T_L - 1` (last index of
        // the deepest fully-internal layer), etc.
        let last_internal = H::last_internal_node(n);
        for i in (0..=last_internal).rev() {
            H::heapify(arr, n, i, logger);
        }
    }
}
