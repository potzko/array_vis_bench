//! Heap sort whose build phase uses quickselect on layer boundaries.
//!
//! Reuses the existing [`HeapSort<H, DH>`] orchestration — only the
//! [`DeepHeapify`] strategy is swapped. The three quickselect-based
//! strategies live in [`super::quick_deep_heapify`]; each is parametrised
//! over a [`HeapPartition`] and a [`PivotSelector`]. The sort family
//! below cross-products `Arity × HeapDirection × QuickDeepHeapify ×
//! HeapPartition × PivotSelector`, with the same `MaxForward` /
//! `MinReverse` direction restriction as classic heap sort (only those
//! two produce ascending output).

combo_codegen::family!(
    type = HeapSort<ArityHeap<{A}, {D}>, {DH}<{HP}, {V}>>,
    uses = [
        "crate::sorts::heap_sort::arity::{Binary, Ternary, Base16, Base256}",
        "crate::sorts::heap_sort::direction::{MinReverse, MaxForward}",
        "crate::sorts::heap_sort::arity_heap::ArityHeap",
        "crate::sorts::heap_sort::heap_sort::HeapSort",
        "crate::sorts::heap_sort::quick_deep_heapify::{SequentialQuickDeepHeapify, RecursivePartialQuickDeepHeapify, StackPartialQuickDeepHeapify}",
        "crate::sorts::heap_sort::heap_partition::{Block, Hoare, Lomuto}",
        "crate::sorts::quick_sorts::pivot_selectors::{FirstElement, LastElement, MedianOfMedians, MedianOfThree, MiddleElement, Ninther}",
    ],
    A: Arity,
    D: HeapDirection,
    DH: QuickDeepHeapify,
    HP: HeapPartition,
    V: PivotSelector,
    name = "heap sort quick build",
    big_o = "O(N log N)",
    stable = false,
    direct_sort = true,
    path = ["heap sorts", "quick build", "{D}", "{A}", "{DH}", "{HP}", "{V}"],
);

combo_codegen::family!(
    type = HeapSort<ArityHeap<{A}, {D}>, {DH}<{DPS}>>,
    uses = [
        "crate::sorts::heap_sort::quick_deep_heapify::StackDualPivotPartialQuickDeepHeapify",
        "crate::sorts::quick_sorts::pivot_selectors::{CombinedSelector, NintherDualPivot}",
    ],
    A: Arity,
    D: HeapDirection,
    DH: DualPivotQuickDeepHeapify,
    DPS: inline [
        ("CombinedSelector<FirstElement, FirstElement>",   "first"),
        ("CombinedSelector<MiddleElement, MiddleElement>", "middle"),
        ("NintherDualPivot",                               "ninther 1/3 + 2/3"),
    ],
    name = "heap sort quick build dual pivot",
    big_o = "O(N log N)",
    stable = false,
    direct_sort = true,
    path = ["heap sorts", "quick build", "dual pivot", "{D}", "{A}", "{DH}", "{DPS}"],
);
