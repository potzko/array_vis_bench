//! Re-export shim. The merge sort family + the standalone-merge
//! registrations live in `merge_sort_lib`. Family TOML declarations in
//! that crate's `Cargo.toml` plus the `standalone_registry` ctors
//! self-register every variant into `array_vis_bench_core::ALGORITHMS`.
//!
//! Sub-module shims below preserve the `crate::sorts::merge_sorts::*`
//! path layout used by `compare_sorts` and benches.

pub mod auxiliary_merge {
    pub use merge_sort_lib::auxiliary_merge::*;
}
pub mod bottom_up {
    pub use merge_sort_lib::bottom_up::*;
}
pub mod naive {
    pub use merge_sort_lib::naive::*;
}
pub mod natural {
    pub use merge_sort_lib::natural::*;
}
pub mod rotation {
    pub use merge_sort_lib::rotation::*;
}
pub mod rotation_merge {
    pub use merge_sort_lib::rotation_merge::*;
}
pub mod top_down {
    pub use merge_sort_lib::top_down::*;
}

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/merge_sorts_combinations.rs"));
}
