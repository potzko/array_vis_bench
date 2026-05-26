//! Event log for sorting algorithms.
//!
//! The [`SortLogger`] trait is the universal callback every sort
//! algorithm in this workspace calls into. Each method records one
//! observable operation (compare, swap, write, allocate auxiliary
//! array, …) without committing to a storage strategy — implementors
//! decide whether to record events for replay, count them for
//! benchmarking, or discard them entirely.
//!
//! Two reference implementations ship here:
//!
//! - [`NoOpLogger`] — zero-sized, every method is empty. Use for
//!   benchmarking when only the underlying sort work should be timed.
//! - [`VisualizerLogger`] — buffers every event as a [`SortLog`] for
//!   later rendering by `sort_vis` into a GIF / MP4.
//!
//! Algorithm crates that want to be benchmarked + visualised should
//! depend on this crate and call logger methods at the points they
//! want to surface. The trait is dyn-compatible, so the same algorithm
//! body works against both monomorphic `NoOpLogger` (in the perf gate)
//! and a `&mut dyn SortLogger<T>` (in the visualiser pipeline).

#![allow(clippy::uninit_vec)]

mod loggers;
mod sort_log;
mod sort_logger;

pub use loggers::{NoOpLogger, VisualizerLogger};
pub use sort_log::SortLog;
pub use sort_logger::SortLogger;
