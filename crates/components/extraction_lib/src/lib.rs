//! How a block merge sort carves its keys and buffer. `partition` holds the
//! `StablePartition` fillers, `RotationPartition` and `SlidingPartition`.

pub mod partition;

pub use partition::{RotationPartition, SlidingPartition};
