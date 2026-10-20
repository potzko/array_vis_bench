//! Comb sort family: the `CombSort` driver and the const-generic
//! `CombSortRatio<NUM, DEN>`, with six shrink ratios declared as components in
//! `comb.spec` and emitted by the catalog through the `CombSortOf<{ratio}>`
//! driver.

pub mod comb_sort;
pub mod comb_sort_ratio;

pub use comb_sort::CombSort;
pub use comb_sort_ratio::{CombRatio, CombSortOf, CombSortRatio};
