# `src`

Source of `quick_heap_sort_lib`. The crate README one level up says what it is for.

- `deferred_quick_heap_sort.rs`: Deferred quick heap sort.
- `heap_extract.rs`: `HeapExtract`, heap-based partition as a `PartitionScheme`.
- `heap_pair.rs`: `HeapAlgorithmPair<DH>`, pick the two opposite-direction `HeapAlgorithm` instances that `crate::heap_partition_core::build_and_converge` drives,...
- `heap_partition_core.rs`: Shared "build two heaps + converge heads" routine.
- `lib.rs`: Quick heap sort family, hybrid quicksort/heapsort built on `heap_sort_lib`'s `ArityHeap` machinery.
- `quick_heap_sort.rs`: Quick heap sort.
