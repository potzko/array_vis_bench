//! Reference [`SortLogger`] implementations: [`NoOpLogger`] and
//! [`VisualizerLogger`]. The `impl … for ()` is kept so anonymous unit
//! values can be threaded through APIs that take `&mut impl SortLogger`.

use crate::sort_log::SortLog;
use crate::sort_logger::SortLogger;

impl<T: Copy + Ord> SortLogger<T> for () {
    #[inline]
    fn log(&mut self, _: SortLog<T>) {}
}

/// Buffers every logged event into an in-memory `Vec` for later
/// rendering. The `sort_vis` crate consumes the resulting `log` to
/// produce a GIF / MP4 of the sort's execution.
///
/// Allocates per push; not appropriate for perf-sensitive paths — use
/// [`NoOpLogger`] there instead.
#[derive(Debug)]
pub struct VisualizerLogger<T: Copy + Ord> {
    /// Carries the element type as a phantom so the same logger can be
    /// monomorphised against the array's actual element type without
    /// holding any real data.
    pub type_ghost: std::marker::PhantomData<T>,
    /// Recorded events in chronological order.
    pub log: Vec<SortLog<T>>,
}

impl<T: Copy + Ord> SortLogger<T> for VisualizerLogger<T> {
    #[inline]
    fn log(&mut self, data: SortLog<T>) {
        self.log.push(data);
    }
}

/// Zero-sized [`SortLogger`] whose methods compile to nothing.
///
/// Use this in the perf gate so the trait/inlining overhead is the
/// only thing being measured. Construct directly: `&mut NoOpLogger`.
#[derive(Debug, Clone, Copy)]
pub struct NoOpLogger;

impl<T: Copy + Ord> SortLogger<T> for NoOpLogger {
    #[inline]
    fn log(&mut self, _: SortLog<T>) {}
}
