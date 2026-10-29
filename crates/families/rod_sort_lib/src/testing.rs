//! Test helpers shared by the crate's unit tests: `Compares`, a logger that
//! counts comparisons and drops every other event.

use sort_logger::{SortLog, SortLogger};

pub(crate) struct Compares(pub(crate) usize);

impl<T: Copy + Ord> SortLogger<T> for Compares {
    fn log(&mut self, event: SortLog<T>) {
        if matches!(event, SortLog::CmpInArr { .. } | SortLog::CmpData { .. } | SortLog::CmpAcrossArrs { .. }) {
            self.0 += 1;
        }
    }
}
