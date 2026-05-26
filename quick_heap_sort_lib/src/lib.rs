//! Quick heap sort family — hybrid quicksort/heapsort built on
//! `heap_sort_lib`'s `ArityHeap` machinery. The classic, quickselect-build,
//! and deferred variants are declared as TOML families in this crate's
//! `Cargo.toml`.

pub mod deferred_quick_heap_sort;
pub mod quick_heap_sort;

pub use deferred_quick_heap_sort::DeferredQuickHeapSort;
pub use quick_heap_sort::QuickHeapSort;
