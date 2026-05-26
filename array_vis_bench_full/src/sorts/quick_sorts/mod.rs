//! Re-export shim. The quick sort family + the standalone-partition
//! registrations live in `quick_sort_lib`. Family TOML declarations in
//! that crate's `Cargo.toml` self-register every variant.

pub mod partitions; // shim re-exporting partition leaves
pub mod pivot_selectors {
    pub use quick_sort_lib::pivot_selectors::*;
}
pub mod quick_sort {
    pub use quick_sort_lib::quick_sort::*;
}
pub mod dual_pivot_quick_sort {
    pub use quick_sort_lib::dual_pivot_quick_sort::*;
}
pub mod deferred_quick_sort {
    pub use quick_sort_lib::deferred_quick_sort::*;
}
pub mod deferred_dual_pivot_quick_sort {
    pub use quick_sort_lib::deferred_dual_pivot_quick_sort::*;
}
pub mod partitions_standalone {
    pub use quick_sort_lib::partitions_standalone::*;
}

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/quick_sorts_combinations.rs"));
}
