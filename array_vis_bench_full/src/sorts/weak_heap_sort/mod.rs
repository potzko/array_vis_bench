//! Re-export shim. `WeakHeapSort<D, R>` lives in `weak_heap_sort_lib`.

pub mod reverse_storage; // already a leaf-shim
pub mod weak_heap_sort {
    pub use weak_heap_sort_lib::weak_heap_sort::*;
}

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/weak_heap_sort_combinations.rs"));
}
