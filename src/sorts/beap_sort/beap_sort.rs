//! Beap sort — classic family.
//!
//! Reuses the existing [`HeapSort<H, DH>`] orchestration with
//! [`BeapHeap<D>`] as the underlying heap. Only `Iterative` DeepHeapify is
//! registered for beap — the `Recursive` strategy's "recurse into each
//! child subtree" pattern double-visits beap nodes through the
//! bi-parental DAG and blows up exponentially. The iterative variant
//! visits each internal node exactly once.

combo_codegen::family!(
    type = HeapSort<BeapHeap<{D}>, {DH}>,
    uses = [
        "crate::sorts::heap_sort::direction::{MinReverse, MaxForward}",
        "crate::sorts::heap_sort::heap_sort::HeapSort",
        "crate::sorts::heap_sort::deep_heapify::Iterative",
        "crate::sorts::beap_sort::beap_heap::BeapHeap",
    ],
    D: HeapDirection,
    DH: inline [
        ("Iterative", "iterative"),
    ],
    name = "beap sort",
    big_o = "O(N sqrt(N))",
    stable = false,
    direct_sort = true,
    path = ["beap sorts", "{D}", "{DH}"],
);
