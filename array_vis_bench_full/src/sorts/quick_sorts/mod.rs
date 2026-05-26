//! Re-export shim. The quick sort family lives in `quick_sort_lib`;
//! the standalone-partition registrations live in
//! `quick_partition_registry`. Family TOML declarations in
//! `quick_sort_lib/Cargo.toml` plus the registry's ctor block
//! self-register every variant.

pub mod partitions; // shim re-exporting partition leaves
pub mod pivot_selectors {
    // Re-export simple pivots from their own leaves (no longer
    // re-exported through `quick_sort_lib::pivot_selectors` — that
    // facade was forcing every consumer to pull every pivot leaf in).
    // `CombinedSelector` and `NintherDualPivot` still live in
    // `quick_sort_lib` because they're foreign-trait-on-local-type for
    // the dual-pivot family.
    pub use array_vis_bench_traits::{DualPivotSelector, PivotSelector};
    pub use pivot_first::FirstElement;
    pub use pivot_last::LastElement;
    pub use pivot_median3::MedianOfThree;
    pub use pivot_median_of_medians::MedianOfMedians;
    pub use pivot_middle::MiddleElement;
    pub use pivot_ninther::Ninther;
    pub use quick_sort_lib::pivot_selectors::{CombinedSelector, NintherDualPivot};
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

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/quick_sorts_combinations.rs"));
}
