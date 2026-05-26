//! Role traits — one per axis a leaf component can implement.
//!
//! Each role module holds exactly the trait definition; concrete
//! implementations live in per-leaf crates (`partition_lomuto`,
//! `rotation_reversal`, …) or, until they're carved out, in their
//! original module in `array_vis_bench`.

pub mod partition;
pub mod pivot;
pub mod quick_select;
pub mod rotation;
pub mod small_sort;

pub use partition::PartitionScheme;
pub use pivot::{DualPivotSelector, PivotSelector};
pub use quick_select::QuickSelect;
pub use rotation::Rotation;
pub use small_sort::{
    insertion_sort_with, DeferredSmallSort, InsertionStrategy, NonTrivialSmallSort,
    SetSizeSmallSort, SmallSort, SmallSortAdapter,
};
