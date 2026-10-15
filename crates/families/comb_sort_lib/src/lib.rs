//! Comb sort family: the `CombSort` driver and the const-generic
//! `CombSortRatio<NUM, DEN>`, with six shrink ratios declared as components in
//! `comb.spec` and emitted by the catalog through the `CombSortOf<{ratio}>`
//! driver. `registration.rs` keeps the older visualiser-only menu
//! registration behind the `self_register` feature, off by default.

pub mod comb_sort;
pub mod comb_sort_ratio;
pub mod registration;

pub use comb_sort::CombSort;
pub use comb_sort_ratio::{CombRatio, CombSortOf, CombSortRatio};
pub use registration::{CombEntry, SortFn, COMB_SEQUENCES};
