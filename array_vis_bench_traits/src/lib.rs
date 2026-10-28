//! Pure-trait crate for the `array_vis_bench` workspace.
//!
//! Holds the surface that leaf-component crates (`partition_lomuto`,
//! `rotation_reversal`, …) need to implement without pulling in the
//! full `array_vis_bench` tree:
//!
//! - [`complexity::Complexity`] — compile-time Big-O class.
//! - [`composable`] — per-axis annotation traits (`HasTimeBounds`,
//!   `HasSpace`, `HasStability`, `PivotQuality`).
//!
//! - [`role`], the role traits (`PartitionScheme`, `PivotSelector`,
//!   `SmallSort`, `Rotation`, `RotationMerge`, `DirectSort`, `QuickSelect`, `Auxiliary`,
//!   `Keys`, `Scratchspace`).
//!
//! Family-local traits such as `GapSequence` and `BranchingStrategy` stay in
//! the crate of their only consumer.

// The `direct_sort!` macro names the logger trait through this path, so a crate
// that invokes it needs no import of its own.
#[doc(hidden)]
pub use sort_logger;

pub mod complexity;
pub mod composable;
pub mod role;

pub use complexity::{Complexity, Special};
pub use composable::{HasSpace, HasStability, HasTimeBounds, PivotQuality};
pub use role::{
    with_partition_scratch, DeferredSmallSort, DirectSort, DualPivotSelector, InsertionStrategy, NoPivot,
    NonTrivialSmallSort, PartitionScheme, PartitionVisitor, PivotInput, PivotSelector, QuickSelect,
    Keys, Rotation, RotationMerge, Scratchspace, ScratchspaceRun, SetSizeSmallSort, SmallSort,
    SmallSortAdapter,
};
pub use role::rotation::reverse;
pub use role::small_sort::{insertion_sort_with, windowed_insertion_sort_with};
