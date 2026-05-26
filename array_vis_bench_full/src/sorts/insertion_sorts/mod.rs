//! Re-export shim. `InsertionSort<S>` lives in `insertion_sort_lib`; its
//! family declaration is in that crate's `Cargo.toml`. The build
//! script's TOML-metadata scanner picks the declaration up automatically
//! when `insertion_sort_lib` is in the dep graph.

pub use insertion_sort_lib::InsertionSort;

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/insertion_sorts_combinations.rs"));
}
