//! Re-export shim. All fun sorts live in `fun_sorts_lib`. Family TOML
//! declarations in that crate's `Cargo.toml` self-register every variant
//! into `array_vis_bench_core::ALGORITHMS`.

pub use fun_sorts_lib::*;

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/fun_sorts_combinations.rs"));
}
