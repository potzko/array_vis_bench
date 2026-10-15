//! Role traits — one per axis a leaf component can implement.
//!
//! Each role module holds exactly the trait definition; concrete
//! implementations live in the leaf crates under `crates/`
//! (`partition_lomuto`, `rotation_reversal`, ...).

pub mod auxiliary;
pub mod partition;
pub mod pivot;
pub mod quick_select;
pub mod rotation;
pub mod scratchspace;
pub mod small_sort;

pub use auxiliary::Auxiliary;
pub use partition::{with_partition_scratch, PartitionScheme, PartitionVisitor};
pub use scratchspace::{Scratchspace, ScratchspaceRun};
pub use pivot::{DualPivotSelector, NoPivot, PivotInput, PivotSelector};
pub use quick_select::QuickSelect;
pub use rotation::Rotation;
pub use small_sort::{
    insertion_sort_with, windowed_insertion_sort_with, DeferredSmallSort, InsertionStrategy,
    NonTrivialSmallSort, SetSizeSmallSort, SmallSort, SmallSortAdapter,
};
