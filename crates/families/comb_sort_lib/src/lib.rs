//! Comb sort family. Two registration paths:
//!
//! - **Visualiser-only** (this crate's `registration.rs`): six per-ratio
//!   closures registered into `COMB_SEQUENCES` + `sort_registry_core`'s
//!   menu tree. These render the gap-shrink animation directly.
//! - **Algorithm** (`[[package.metadata.array_vis_bench.families]]` in
//!   this crate's `Cargo.toml`): the cross-product over `CombRatio`
//!   components — each generating an `ALGORITHMS` entry for
//!   `CombSortRatio<NUM, DEN>` via the `SortAlgo` trait route.

pub mod comb_sort;
pub mod comb_sort_ratio;
pub mod registration;

pub use comb_sort::CombSort;
pub use comb_sort_ratio::{CombRatio, CombSortOf, CombSortRatio};
pub use registration::{CombEntry, SortFn, COMB_SEQUENCES};
