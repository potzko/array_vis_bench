//! Smooth sort, `SmoothSort<D, S, F, B>`, over a heap direction `D`, a tree
//! shape `S` (`Leonardo` or `Perfect`, in `shape.rs`), a sift `F` (`Pairs` or
//! `BottomUp`, in `sift.rs`) and a build `B` (`Insert`, in `build.rs`). `heap.rs` holds the operations on a forest and
//! `tree.rs` the moves and comparisons they are made of. The family is
//! declared in `smooth.spec` and built by `smooth.avbs`.

pub mod build;
pub mod heap;
pub mod shape;
pub mod sift;
pub mod smooth_sort;
pub mod tree;

#[cfg(test)]
mod tests;

pub use build::{Build, Insert};
pub use shape::{Forest, Leonardo, Perfect, Shape};
pub use sift::{BottomUp, Pairs, Sift};
pub use smooth_sort::SmoothSort;
