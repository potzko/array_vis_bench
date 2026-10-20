# `quick_heap_sort_lib`

Quick heap sort: a hybrid that builds a max heap on the left and a min heap on the right and converges by head swapping. Classic and deferred drivers over heap arity and small sort, the heap extract partitions quick select uses, and the recursive set quick select family bounded by finite().

Its catalog components are declared in `quick_heap.spec`.

Query `quick_heap.avbs` builds: `QuickHeapSortsClassic`, `QuickHeapSortsDeferred`, `SetQuickHeapSorts`.

`tests/` holds the crate's integration tests.
