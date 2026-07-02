//! Quick heap sort family — hybrid quicksort/heapsort built on
//! `heap_sort_lib`'s `ArityHeap` machinery. The classic, quickselect-build,
//! and deferred variants are declared as TOML families in this crate's
//! `Cargo.toml`.

pub mod deferred_quick_heap_sort;
pub mod heap_extract;
pub mod heap_pair;
pub mod heap_partition_core;
pub mod quick_heap_sort;

pub use deferred_quick_heap_sort::DeferredQuickHeapSort;
pub use heap_extract::HeapExtract;
pub use heap_pair::{AryPair, BeapPair, HeapAlgorithmPair};
pub use quick_heap_sort::QuickHeapSort;

/// A binary `QuickHeapSort` with no small-sort cutoff, leaving only the
/// deep-heapify `DH` open. It exists to give the `finite()`-bounded recursive
/// set-quick-select family a DISTINCT type head (the plain `QuickHeapSort` head is
/// already owned by the classic family, so the AVBS query could not reference this
/// one unambiguously). As an alias it inherits every `QuickHeapSort` impl, so
/// `SetQuickHeapSort<SequentialSet<RecursiveQuickSelect<HeapExtract<AryPair<Binary>,
/// Iterative>>>>` compiles and sorts (see tests/correctness.rs).
pub type SetQuickHeapSort<DH> =
    QuickHeapSort<heap_sort_lib::arity::Binary, DH, small_sort_basic::NoSmallSort>;
