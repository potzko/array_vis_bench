//! Re-export shim. `SortAlgo` lives in `array_vis_bench_traits` so leaf
//! crates can implement it without depending on the full
//! `array_vis_bench` tree.

pub use array_vis_bench_traits::sort_traits::SortAlgo;
