//! Re-export shim. The bubble sort family — BubbleSort, ShakerSort,
//! BubbleSortRecursive, OddEvenBubbleSort<S> — lives in
//! `bubble_sort_lib`. The first three self-register via inline
//! `sort_family!` calls; the cross-product over `NonTrivialSmallSort`
//! for OddEvenBubbleSort comes from the family TOML in
//! `bubble_sort_lib/Cargo.toml`.

pub use bubble_sort_lib::{BubbleSort, BubbleSortRecursive, OddEvenBubbleSort, ShakerSort};

pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/bubble_sorts_combinations.rs"));
}
