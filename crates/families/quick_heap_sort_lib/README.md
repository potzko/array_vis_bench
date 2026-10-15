# `quick_heap_sort_lib`

Quick heap sort: a hybrid that builds a max heap on the left and a min heap on the right and converges by head swapping. Classic and deferred drivers over heap arity and small sort, the heap extract partitions quick select uses, and the recursive set quick select family bounded by finite().

Catalog fragment `quick_heap.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `qhs_classic` | `QuickHeapSort<{arity}, Iterative, {small}>` | `quick heap sort<arity: {arity}, small: {small}>` | Sort | Sort |
| `qhs_deferred` | `DeferredQuickHeapSort<{arity}, {dss}>` | `quick heap sort deferred<arity: {arity}, small: {dss}>` | Sort | Sort |
| `qs_part_he_bin_it` | `HeapExtract<AryPair<Binary>, Iterative>` | `heap extract: binary iterative` | HeapExtractPart SelectPartition |  |
| `qs_part_he_bin_re` | `HeapExtract<AryPair<Binary>, Recursive>` | `heap extract: binary recursive` | HeapExtractPart SelectPartition |  |
| `qs_part_he_ter_it` | `HeapExtract<AryPair<Ternary>, Iterative>` | `heap extract: ternary iterative` | HeapExtractPart SelectPartition |  |
| `qs_part_he_ter_re` | `HeapExtract<AryPair<Ternary>, Recursive>` | `heap extract: ternary recursive` | HeapExtractPart SelectPartition |  |
| `rdh_iter` | `Iterative` | `iterative` | RecDeepHeapify |  |
| `rdh_rec` | `Recursive` | `recursive` | RecDeepHeapify |  |
| `rdh_set` | `SequentialSet<{qs}>` | `set<{qs}>` | RecDeepHeapify SetDeepHeapify |  |
| `rsel` | `SetSelect<{part}>` | `select<{part}>` | RecSelect |  |
| `hx_open` | `HeapExtract<AryPair<Binary>, {dh}>` | `heap extract: binary<{dh}>` | RecSelPart |  |
| `set_quick_heap_sort` | `SetQuickHeapSort<{dh}>` | `set quick heap sort<{dh}>` | Sort | Sort |

Query `quick_heap.avbs` builds: `QuickHeapSortsClassic`, `QuickHeapSortsDeferred`, `SetQuickHeapSorts`.

`tests/` holds the crate's integration tests.
