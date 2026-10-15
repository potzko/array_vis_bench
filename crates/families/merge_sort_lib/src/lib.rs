//! Merge sort family — classic (TopDown, BottomUp, Naive, Natural) and
//! rotation-based merge sorts.
//!
//! The sort families and the first-class merge kinds are declared in
//! `merge.spec` and built by `merge.avbs`; the catalog emits every variant.

pub mod auxiliary_merge;
pub mod bottom_up;
pub mod naive;
pub mod natural;
pub mod rotation;
pub mod rotation_merge;
pub mod top_down;
mod utils;

pub use auxiliary_merge::{AuxMerge, FullCopyAuxMerge, HalfCopyAuxMerge};
pub use bottom_up::BottomUpMergeSort;
pub use naive::NaiveMergeSort;
pub use natural::NaturalMergeSort;
pub use rotation::{BottomUpRotationMergeSort, TopDownRotationMergeSort};
pub use rotation_merge::{NaiveRotationMerge, RotationMerge, SmallerSideRotationMerge};
pub use top_down::TopDownMergeSort;
