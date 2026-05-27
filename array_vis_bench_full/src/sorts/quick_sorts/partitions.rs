//! Partition schemes for quicksort and standalone-partition algorithms.
//!
//! Every variant has been carved out into its own leaf crate
//! (`partition_lomuto`, `partition_hoare`, `partition_block`,
//! `partition_three_way`, `partition_moving_pivot`,
//! `partition_moving_pivot_v3`). This file re-exports them under the
//! historical `super::partitions::…` paths so the QuickSort family!
//! `uses` block and every other consumer keeps resolving unchanged.

// `PartitionScheme` lives in the `array_vis_bench_traits` crate.
pub use array_vis_bench_traits::PartitionScheme;

pub use partition_block::Block;
pub use partition_hoare::LeftRightPartition;
pub use partition_lomuto::LeftLeftPartition;
pub use partition_moving_pivot::MovingPivot;
pub use partition_moving_pivot_v3::MovingPivotV3;
pub use partition_three_way::ThreeWay;
