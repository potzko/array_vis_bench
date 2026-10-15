//! Beap sort family — `BeapHeap<D>` plugged into the generic
//! `HeapSort<H, DH>` orchestration from `heap_sort_lib`. The families are
//! declared in `beap.spec` and built by `beap.avbs`.

pub mod beap_heap;
pub use beap_heap::BeapHeap;

pub mod spec_drivers;
pub use spec_drivers::{BeapSortClassicOf, BeapSortDualOf, BeapSortQuickOf};
