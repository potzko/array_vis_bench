//! Generic heap sort: build heap, then repeatedly swap the root past the
//! shrinking heap and re-heapify.
//!
//! `HeapSort<H>` is layout/compare/arity-agnostic — all of that lives in
//! `H`. The sort_family! call below cross-products `Arity` × `HeapDirection`
//! to register every ascending-producing variant. `HeapDirection` only
//! has component! markers on `MinReverse` and `MaxForward`, so the other
//! two directions (which would sort to descending) are excluded.

use std::marker::PhantomData;

use super::heap::Heap;
use crate::traits::log_traits::SortLogger;

pub struct HeapSort<H: Heap> {
    _phantom: PhantomData<H>,
}

impl<H: Heap> HeapSort<H> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        // Build the heap from scratch (recursive deep build from root).
        H::deep_heapify(arr, n, 0, logger);
        // Extract: swap root with last logical element, shrink, re-heapify root.
        for heap_size in (2..=n).rev() {
            H::swap(arr, 0, heap_size - 1, logger);
            H::heapify(arr, heap_size - 1, 0, logger);
        }
    }
}

combo_codegen::sort_family!(
    type = HeapSort<ArityHeap<{A}, {D}>>,
    uses = [
        "crate::sorts::heap_sort::arity::{Binary, Ternary, Base16, Base256}",
        "crate::sorts::heap_sort::direction::{MinReverse, MaxForward}",
        "crate::sorts::heap_sort::arity_heap::ArityHeap",
        "crate::sorts::heap_sort::heap_sort::HeapSort",
    ],
    A: Arity,
    D: HeapDirection,
    name = "heap sort",
    big_o = "O(N log N)",
    stable = false,
    direct_sort = true,
    path = ["heap sorts", "{D}", "{A}"],
);
