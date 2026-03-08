use crate::sort_log::SortLog;
use crate::sort_logger::SortLogger;

impl<T: Copy + Ord> SortLogger<T> for () {
    #[inline]
    fn log(&mut self, _: SortLog<T>) {}
}

#[derive(Debug)]
pub struct VisualizerLogger<T: Copy + Ord> {
    pub type_ghost: std::marker::PhantomData<T>,
    pub log: Vec<SortLog<T>>,
}

impl<T: Copy + Ord> SortLogger<T> for VisualizerLogger<T> {
    #[inline]
    fn log(&mut self, data: SortLog<T>) {
        self.log.push(data);
    }
}

/// A no-operation logger that does nothing when logging
#[derive(Debug, Clone, Copy)]
pub struct NoOpLogger;

impl<T: Copy + Ord> SortLogger<T> for NoOpLogger {
    #[inline]
    fn log(&mut self, _: SortLog<T>) {}
}
