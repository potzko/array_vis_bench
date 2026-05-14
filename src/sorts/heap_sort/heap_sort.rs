//! Generic n-ary heap sort.
//!
//! `HeapSort<H, DH>` is layout/compare/arity-agnostic — all of that lives
//! in `H` — and also deep-heapify-strategy-agnostic via `DH`. The actual
//! build-then-extract loop is the default implementation on
//! [`HeapAlgorithm`]; this type only supplies the primitives.
//!
//! The `family!` call below cross-products `Arity` × `HeapDirection` ×
//! `DeepHeapify` to register every ascending-producing variant.
//! `HeapDirection` only has component! markers on `MinReverse` and
//! `MaxForward`, so the other two directions (which would sort to
//! descending) are excluded.

use std::marker::PhantomData;

use super::deep_heapify::DeepHeapify;
use super::heap::Heap;
use super::heap_algorithm::HeapAlgorithm;
use crate::traits::log_traits::SortLogger;

pub struct HeapSort<H: Heap, DH: DeepHeapify> {
    _phantom: PhantomData<(H, DH)>,
}

impl<H: Heap, DH: DeepHeapify> HeapAlgorithm for HeapSort<H, DH> {
    type State = ();

    #[inline(always)]
    fn new_state<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_n: usize, _logger: &mut U) {}

    #[inline(always)]
    fn root_phys(n: usize) -> usize {
        H::phys(0, n)
    }

    #[inline(always)]
    fn build<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        _state: &mut (),
        logger: &mut U,
    ) {
        DH::deep_heapify::<H, T, U>(arr, logger);
    }

    #[inline(always)]
    fn swap_root_to_end<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        _state: &mut (),
        heap_size: usize,
        logger: &mut U,
    ) {
        H::swap(arr, 0, heap_size - 1, logger);
    }

    #[inline(always)]
    fn push_down<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        _state: &mut (),
        heap_size: usize,
        logger: &mut U,
    ) {
        H::heapify(arr, heap_size, 0, logger);
    }
}

impl<H: Heap, DH: DeepHeapify> HeapSort<H, DH> {
    /// Inherent thin delegate so `<HeapSort<...>>::sort(arr, logger)` keeps
    /// working from `family!`-generated code without needing the
    /// `HeapAlgorithm` trait in scope at the call site.
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        <Self as HeapAlgorithm>::sort(arr, logger)
    }
}

combo_codegen::family!(
    type = HeapSort<ArityHeap<{A}, {D}>, {DH}>,
    uses = [
        "crate::sorts::heap_sort::arity::{Binary, Ternary, Base16, Base256}",
        "crate::sorts::heap_sort::direction::{MinReverse, MaxForward}",
        "crate::sorts::heap_sort::arity_heap::ArityHeap",
        "crate::sorts::heap_sort::deep_heapify::{Recursive, Iterative}",
        "crate::sorts::heap_sort::heap_sort::HeapSort",
    ],
    A: Arity,
    D: HeapDirection,
    DH: DeepHeapify,
    name = "heap sort",
    big_o = "O(N log N)",
    stable = false,
    direct_sort = true,
    path = ["heap sorts", "{D}", "{A}", "{DH}"],
);
