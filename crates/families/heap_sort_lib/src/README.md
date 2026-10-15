# `src`

Source of `heap_sort_lib`. The crate README one level up says what it is for.

- `arity.rs`: Heap arity, number of children per heap node.
- `arity_heap.rs`: N-ary heap parameterized by arity and direction (compare + layout).
- `compare.rs`: Heap comparison, selects min-heap vs max-heap ordering.
- `deep_heapify.rs`: Top-level deep-heapify strategy, how to turn the whole array into a heap before extraction starts.
- `direction.rs`: Heap direction, pairs a `Compare` with a `Layout`.
- `heap.rs`: Heap traits, split into a minimal *layout* surface and a richer *operations* surface on top of it.
- `heap_algorithm.rs`: High-level abstraction for heap-based sorting algorithms.
- `heap_partition.rs`: Heap-aware partitions for the quickselect-based deep heapify.
- `heap_sort.rs`: Generic n-ary heap sort + the family marker.
- `heap_sort_quick_build.rs`: Heap sort whose build phase uses quickselect on layer boundaries.
- `layout.rs`: Heap layout, maps logical heap indices to physical array positions.
- `lib.rs`: Heap sort family, `HeapSort<HA>` marker + the `NaryHeapSort<H, DH>` stateless-heap adapter + the supporting trait layers (`HeapLayout`, `Heap`,...
- `quick_deep_heapify.rs`: Build-the-heap-via-quickselect strategies for `DeepHeapify`.
- `set_quick_select.rs`: `SetQuickSelect`, multi-position quickselect contract for heap building.
- `spec_drivers.rs`: Spec-system driver wrappers with distinct Rust type-heads.
