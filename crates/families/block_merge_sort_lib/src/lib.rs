//! Block merge sort, `BlockMergeSort<Buffer, Keys>`. The merge machinery is in
//! `stash` (where the left run waits during a local merge) and `rolling` (the
//! block merge itself, which rolls the blocks of one run through the other).

pub mod rolling;
pub mod stash;

#[cfg(test)]
mod test_util;
