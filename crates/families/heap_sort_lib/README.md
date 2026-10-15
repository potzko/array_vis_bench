# `heap_sort_lib`

Heap sort family: the layered heap machinery (layout, arity, direction, compare, deep heapify, heap partition), the classic, quick build and dual build drivers, and the set quick select heap family that exercises finite().

Catalog fragment `heap.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `heap_arity_binary` | `Binary` | `binary` | HeapArity |  |
| `heap_arity_ternary` | `Ternary` | `ternary` | HeapArity |  |
| `heap_arity_16` | `Base16` | `16-ary` | HeapArity |  |
| `heap_arity_256` | `Base256` | `256-ary` | HeapArity |  |
| `heap_dir_min_reverse` | `MinReverse` | `min reverse` | HeapDir |  |
| `heap_dir_max_forward` | `MaxForward` | `max forward` | HeapDir |  |
| `heap_part_left_left` | `LeftLeftPartition` | `left-left pointer` | HeapPart |  |
| `heap_part_left_right` | `LeftRightPartition` | `left-right pointer` | HeapPart |  |
| `heap_part_block` | `Block` | `block` | HeapPart |  |
| `heap_dh_iterative` | `Iterative` | `iterative` | DeepHeapifyClassic |  |
| `heap_dh_recursive` | `Recursive` | `recursive` | DeepHeapifyClassic |  |
| `heap_qdh_sequential` | `SequentialQuickDeepHeapify<{hp}, {v}>` | `sequential<part: {hp}, pivot: {v}>` | DeepHeapifyQuick |  |
| `heap_qdh_recursive_partial` | `RecursivePartialQuickDeepHeapify<{hp}, {v}>` | `recursive partition<part: {hp}, pivot: {v}>` | DeepHeapifyQuick |  |
| `heap_qdh_stack_partial` | `StackPartialQuickDeepHeapify<{hp}, {v}>` | `stack partition<part: {hp}, pivot: {v}>` | DeepHeapifyQuick |  |
| `heap_dps_first` | `StackDualPivotPartialQuickDeepHeapify<CombinedSelector<FirstElement, FirstElement>>` | `stack dual-pivot partition<first>` | DeepHeapifyDual |  |
| `heap_dps_middle` | `StackDualPivotPartialQuickDeepHeapify<CombinedSelector<MiddleElement, MiddleElement>>` | `stack dual-pivot partition<middle>` | DeepHeapifyDual |  |
| `heap_dps_ninther` | `StackDualPivotPartialQuickDeepHeapify<NintherDualPivot>` | `stack dual-pivot partition<ninther 1/3 + 2/3>` | DeepHeapifyDual |  |
| `heap_sort_classic` | `HeapSortClassicOf<{a}, {d}, {dh}>` | `heap sort<arity: {a}, dir: {d}, build: {dh}>` | Sort | Sort |
| `heap_sort_quick_build` | `HeapSortQuickBuildOf<{a}, {d}, {dh}>` | `heap sort quick build<arity: {a}, dir: {d}, build: {dh}>` | Sort | Sort |
| `heap_sort_dual_build` | `HeapSortDualBuildOf<{a}, {d}, {dh}>` | `heap sort quick build dual pivot<arity: {a}, dir: {d}, build: {dh}>` | Sort | Sort |
| `weak_heap_sort` | `HeapSort<WeakHeapSort<{d}, {r}>>` | `weak heap sort<dir: {d}, storage: {r}>` | Sort | Sort |
| `beap_dh_iterative` | `Iterative` | `iterative` | BeapDeepHeapifyClassic |  |
| `beap_qdh_sequential` | `SequentialQuickDeepHeapify<{hp}, {v}>` | `sequential<part: {hp}, pivot: {v}>` | BeapDeepHeapifyQuick |  |
| `beap_qdh_recursive_partial` | `RecursivePartialQuickDeepHeapify<{hp}, {v}>` | `recursive partition<part: {hp}, pivot: {v}>` | BeapDeepHeapifyQuick |  |
| `beap_qdh_stack_partial` | `StackPartialQuickDeepHeapify<{hp}, {v}>` | `stack partition<part: {hp}, pivot: {v}>` | BeapDeepHeapifyQuick |  |
| `beap_dpqdh_stack_dual` | `StackDualPivotPartialQuickDeepHeapify<{dps}>` | `stack dual-pivot partition<dps: {dps}>` | BeapDeepHeapifyDual |  |

Query `heap.avbs` builds: `HeapSorts`, `HeapSortsQuickBuild`, `HeapSortsDualBuild`.

`tests/` holds the crate's integration tests.
