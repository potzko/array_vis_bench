//! Beap sort family — `BeapHeap<D>` plugged into the generic
//! `HeapSort<H, DH>` orchestration from `heap_sort_lib`. Family
//! declarations are in this crate's `Cargo.toml`.

pub mod beap_heap;
pub use beap_heap::BeapHeap;
