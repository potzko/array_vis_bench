//! Re-export shim. `QuickHeapSort<A, DH, SS>` and
//! `DeferredQuickHeapSort<A, DSS>` live in `quick_heap_sort_lib`.

pub mod deferred_quick_heap_sort {
    pub use quick_heap_sort_lib::deferred_quick_heap_sort::*;
}
pub mod quick_heap_sort {
    pub use quick_heap_sort_lib::quick_heap_sort::*;
}

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/quick_heap_sort_combinations.rs"));
}
