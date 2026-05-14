//! Beap sort with quickselect-based heap construction.
//!
//! Same shape as [`crate::sorts::heap_sort::heap_sort_quick_build`] but
//! with [`BeapHeap<D>`] in place of `ArityHeap`. The
//! [`QuickDeepHeapify`] strategies and [`HeapPartition`] impls don't care
//! whether the heap is binary, n-ary, or beap — they reach the structure
//! through `H::layer_boundaries` and `H::phys`, which `BeapHeap`
//! implements via triangular-number math.
//!
//! Pivot and partition axes are kept aligned with the quick-heap-sort
//! family's "quick build" registration to avoid menu sprawl.

combo_codegen::family!(
    type = HeapSort<BeapHeap<{D}>, {DH}<{HP}, {V}>>,
    uses = [
        "crate::sorts::heap_sort::direction::{MinReverse, MaxForward}",
        "crate::sorts::heap_sort::heap_sort::HeapSort",
        "crate::sorts::heap_sort::quick_deep_heapify::{RecursivePartialQuickDeepHeapify, SequentialQuickDeepHeapify, StackPartialQuickDeepHeapify}",
        "crate::sorts::heap_sort::heap_partition::{Block, Hoare, Lomuto}",
        "crate::sorts::quick_sorts::pivot_selectors::{FirstElement, MedianOfMedians, MedianOfThree, MiddleElement}",
        "crate::sorts::beap_sort::beap_heap::BeapHeap",
    ],
    D: HeapDirection,
    DH: QuickDeepHeapify,
    HP: HeapPartition,
    V: inline [
        ("FirstElement",    "first"),
        ("MiddleElement",   "middle"),
        ("MedianOfThree",   "median of 3"),
        ("MedianOfMedians", "median of medians"),
    ],
    name = "beap sort quick build",
    big_o = "O(N sqrt(N))",
    stable = false,
    direct_sort = true,
    path = ["beap sorts", "quick build", "{D}", "{DH}", "{HP}", "{V}"],
);

combo_codegen::family!(
    type = HeapSort<BeapHeap<{D}>, {DH}<{DPS}>>,
    uses = [
        "crate::sorts::heap_sort::quick_deep_heapify::StackDualPivotPartialQuickDeepHeapify",
        "crate::sorts::quick_sorts::pivot_selectors::{CombinedSelector, NintherDualPivot}",
    ],
    D: HeapDirection,
    DH: DualPivotQuickDeepHeapify,
    DPS: inline [
        ("CombinedSelector<FirstElement, FirstElement>",   "first"),
        ("CombinedSelector<MiddleElement, MiddleElement>", "middle"),
        ("NintherDualPivot",                               "ninther 1/3 + 2/3"),
    ],
    name = "beap sort quick build dual pivot",
    big_o = "O(N sqrt(N))",
    stable = false,
    direct_sort = true,
    path = ["beap sorts", "quick build", "dual pivot", "{D}", "{DH}", "{DPS}"],
);
