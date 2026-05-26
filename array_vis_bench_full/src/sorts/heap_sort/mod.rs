//! Re-export shim. The heap sort family + all supporting traits
//! (`Heap`, `HeapAlgorithm`, `Compare`, `Layout`, `Direction`, etc.)
//! live in `heap_sort_lib`. The TOML family declarations in that crate's
//! `Cargo.toml` self-register the (algorithm × component) cross product
//! into `array_vis_bench_core::ALGORITHMS`.
//!
//! The sub-module shims below preserve the `crate::sorts::heap_sort::*`
//! path layout used by other sorts (quick_heap_sort, beap_sort,
//! weak_heap_sort) so their imports keep resolving without per-file
//! edits.

pub mod arity {
    pub use heap_sort_lib::arity::*;
}
pub mod arity_heap {
    pub use heap_sort_lib::arity_heap::*;
}
pub mod compare {
    pub use heap_sort_lib::compare::*;
}
pub mod deep_heapify {
    pub use heap_sort_lib::deep_heapify::*;
}
pub mod direction {
    pub use heap_sort_lib::direction::*;
}
pub mod heap {
    pub use heap_sort_lib::heap::*;
}
pub mod heap_algorithm {
    pub use heap_sort_lib::heap_algorithm::*;
}
pub mod heap_partition {
    pub use heap_sort_lib::heap_partition::*;
}
pub mod heap_sort {
    pub use heap_sort_lib::heap_sort::*;
}
pub mod layout {
    pub use heap_sort_lib::layout::*;
}
pub mod quick_deep_heapify {
    pub use heap_sort_lib::quick_deep_heapify::*;
}

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/heap_sort_combinations.rs"));
}
