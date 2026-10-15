# `src`

Source of `beap_sort_lib`. The crate README one level up says what it is for.

- `beap_heap.rs`: Beap (bi-parental heap), triangular-layout heap.
- `lib.rs`: Beap sort family, `BeapHeap<D>` plugged into the generic `HeapSort<H, DH>` orchestration from `heap_sort_lib`.
- `spec_drivers.rs`: Spec-system driver wrappers for the beap sort families.
