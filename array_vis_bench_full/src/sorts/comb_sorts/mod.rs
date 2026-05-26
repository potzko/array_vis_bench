//! Re-export shim. The comb-sort algorithm + visualiser-side
//! registration + the per-ratio family declaration live in
//! `comb_sort_lib`; the `CombRatio` components live in
//! `comb_ratio_components`. Together they self-register the full
//! cross-product into `array_vis_bench_core::ALGORITHMS` when both
//! crates are in the dep graph.

pub use comb_sort_lib::{CombEntry, CombSort, CombSortRatio, COMB_SEQUENCES};

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/comb_sorts_combinations.rs"));
}
