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
//! The role traits (`PartitionScheme`, `PivotSelector`, `Rotation`,
//! `SmallSort`, `BranchingStrategy`, `GapSequence`) currently still live
//! in their original modules in `array_vis_bench`; they'll migrate here
//! incrementally in subsequent phases as each leaf is carved out.

pub mod complexity;
pub mod composable;
pub mod role;
pub mod sort_traits;

pub use complexity::{Complexity, Special};
pub use composable::{HasSpace, HasStability, HasTimeBounds, PivotQuality};
pub use role::{
    DeferredSmallSort, DualPivotSelector, InsertionStrategy, NonTrivialSmallSort, PartitionScheme,
    PivotSelector, QuickSelect, Rotation, SetSizeSmallSort, SmallSort, SmallSortAdapter,
};
pub use role::rotation::reverse;
pub use role::small_sort::insertion_sort_with;
pub use sort_traits::SortAlgo;
