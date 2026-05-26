//! Merge sort family — classic (TopDown, BottomUp, Naive, Natural) and
//! rotation-based merge sorts, plus the standalone `Category::Merge`
//! registrations for the 22 rotation merges + 2 auxiliary merges.
//!
//! Cross-product sort families live in
//! `merge_sort_lib/Cargo.toml`'s `[[package.metadata.array_vis_bench.families]]`
//! blocks; the standalone merge registrations are macro-generated in
//! [`standalone_registry`].

pub mod auxiliary_merge;
pub mod bottom_up;
pub mod naive;
pub mod natural;
pub mod rotation;
pub mod rotation_merge;
pub mod standalone_registry;
pub mod top_down;
mod utils;

pub use auxiliary_merge::{AuxMerge, FullCopyAuxMerge, HalfCopyAuxMerge};
pub use bottom_up::BottomUpMergeSort;
pub use naive::NaiveMergeSort;
pub use natural::NaturalMergeSort;
pub use rotation::{BottomUpRotationMergeSort, TopDownRotationMergeSort};
pub use rotation_merge::{NaiveRotationMerge, RotationMerge, SmallerSideRotationMerge};
pub use top_down::TopDownMergeSort;
