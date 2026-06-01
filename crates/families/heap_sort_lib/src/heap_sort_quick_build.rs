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
//!
//! Family declarations (single-pivot + dual-pivot) live in
//! `heap_sort_lib/Cargo.toml`.
