//! Smooth sort, `SmoothSort<D, S, F, R, B>`, over a heap direction `D`, a tree
//! shape `S` (`Leonardo`, `Perfect` or `Narayana`, in `shape.rs`), a sift `F`
//! (`Pairs`, `BottomUp` or `BinarySearch`, in `sift.rs`), the order of the
//! roots `R` (`SortedRoots` or `ScannedRoots`, in `roots.rs`) and a build `B`
//! (`Insert`, `LazyInsert` or `Heapify`, in `build.rs`). `heap.rs` holds the
//! operations on a forest and `tree.rs` the moves and comparisons they are made
//! of. The family is declared in `smooth.spec` and built by `smooth.avbs`.

pub mod build;
pub mod heap;
pub mod roots;
pub mod shape;
pub mod sift;
pub mod smooth_sort;
pub mod tree;

#[cfg(test)]
mod tests;

pub use build::{Build, Heapify, Insert, LazyInsert};
pub use roots::{Roots, ScannedRoots, SortedRoots};
pub use shape::{Forest, Leonardo, Narayana, Perfect, Shape};
pub use sift::{BinarySearch, BottomUp, Pairs, Sift};
pub use smooth_sort::SmoothSort;
