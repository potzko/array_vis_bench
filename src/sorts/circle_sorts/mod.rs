//! Circle sorts.
//!
//! # How circle sort works
//!
//! A single circle-sort *pass* over a range `[start, end]` compares the
//! outermost pair (`arr[start]` vs `arr[end]`), then the next pair inward,
//! and so on, swapping whenever the right element is smaller.  This "wraps
//! around" like a circle — hence the name.  The full sort repeats passes
//! until no swap occurs, guaranteeing convergence.
//!
//! # Two families, two abstractions
//!
//! ## Recursive — abstracted over *ordering* (`orderings.rs`)
//!
//! The recursive family also splits the range at the midpoint and sorts each
//! half.  At each recursion level there are three operations: `circle_pass`,
//! `recurse_left`, `recurse_right`.  The [`RecursiveOrder`] trait abstracts
//! over *which order* these three operations run.  Four orderings are
//! provided: `PreOrder`, `LeftMidRight`, `RightMidLeft`, and `PostOrder`.
//! A shaker variant alternates orderings with depth and is implemented
//! separately.
//!
//! ## Bottom-up — abstracted over *traversal direction* (`directions.rs`)
//!
//! The bottom-up family avoids recursion by iterating over all power-of-two
//! circle sizes explicitly.  The [`BottomUpDirection`] trait abstracts over
//! *in which order* those sizes are visited within one outer iteration.
//! Four directions are provided: `Decreasing`, `Increasing`, `ShakerDecInc`,
//! and `ShakerIncDec`.
//!
//! [`RecursiveOrder`]: orderings::RecursiveOrder
//! [`BottomUpDirection`]: directions::BottomUpDirection

pub mod circle_sort_bottom_up;
pub mod circle_sort_recursive;
pub mod circle_sort_shaker_recursive;
pub mod combinations;
pub mod directions;
pub mod finishing;
pub mod orderings;
pub mod sequences;
