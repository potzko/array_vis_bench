//! Re-export shim. `BeapHeap<D>` lives in `beap_sort_lib`; classic +
//! quickselect-build family declarations are in that crate's
//! `Cargo.toml`.

pub mod beap_heap {
    pub use beap_sort_lib::beap_heap::*;
}

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/beap_sort_combinations.rs"));
}
