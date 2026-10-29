//! Timing an entry's run with the input built outside the clock. `batch_size`
//! says how many runs to time back to back so a tiny run is not lost in the
//! timer's resolution, without the batch's inputs outgrowing memory, and
//! `time_runs` builds that many inputs, times the runs and returns the time
//! of one. The benchmark and the sweep both time through here.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use sort_logger::NoOpLogger;

use crate::bench_registry::{find_input, AlgorithmEntry, RunConfig};

/// A batch of runs is at least this long, well past the clock's resolution.
pub const BATCH_FLOOR: Duration = Duration::from_micros(50);

/// The most runs one batch times, so a batch of microsecond runs still ends
/// within the floor's order of magnitude.
pub const BATCH_CAP: usize = 64;

/// The most elements a batch's inputs hold together, 8 MiB of `usize`. Every
/// input of a batch is built before the clock starts, so an entry whose run
/// stays tiny at a large size would otherwise hold the cap's worth of copies.
pub const BATCH_ELEMENTS: usize = 1 << 20;

/// How many runs of size `n` to time back to back when one takes `one`:
/// enough to reach the floor, at most the cap and at most what fits in
/// `BATCH_ELEMENTS`, but always one.
pub fn batch_size(one: Duration, n: usize) -> usize {
    let room = (BATCH_ELEMENTS / n.max(1)).clamp(1, BATCH_CAP);
    if one.is_zero() { return room; }
    let runs = (BATCH_FLOOR.as_nanos() + one.as_nanos() - 1) / one.as_nanos();
    runs.clamp(1, room as u128) as usize
}

/// The time of one run of `entry` on the category's `input` at size `n`. One
/// input is built per seed before the clock starts, the runs go back to back
/// under the no op logger, and the total is divided by the count. `None`
/// when a run panicked.
pub fn time_runs(entry: &AlgorithmEntry, input: &str, n: usize, seeds: &[u64]) -> Option<Duration> {
    let generate = find_input(entry.category, input).generate;
    let mut inputs: Vec<(Vec<usize>, usize)> =
        seeds.iter().map(|&seed| generate(&RunConfig { size: n, seed })).collect();
    let start = Instant::now();
    let ran = catch_unwind(AssertUnwindSafe(|| {
        for (arr, index) in &mut inputs { (entry.run)(arr, *index, &mut NoOpLogger); }
    }));
    ran.ok().map(|()| start.elapsed() / seeds.len().max(1) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_batch_reaches_the_floor_within_the_cap() {
        assert_eq!(batch_size(Duration::from_micros(100), 1000), 1);
        assert_eq!(batch_size(Duration::from_micros(50), 1000), 1);
        assert_eq!(batch_size(Duration::from_micros(20), 1000), 3);
        assert_eq!(batch_size(Duration::from_nanos(1000), 1000), 50);
        assert_eq!(batch_size(Duration::from_nanos(100), 1000), BATCH_CAP);
        assert_eq!(batch_size(Duration::ZERO, 1000), BATCH_CAP);
    }

    #[test]
    fn a_batch_of_large_inputs_holds_at_most_the_element_ceiling() {
        let tiny = Duration::from_nanos(10);
        assert_eq!(batch_size(tiny, BATCH_ELEMENTS / 32), 32);
        assert_eq!(batch_size(tiny, BATCH_ELEMENTS), 1);
        assert_eq!(batch_size(tiny, BATCH_ELEMENTS * 64), 1);
        assert_eq!(batch_size(Duration::ZERO, BATCH_ELEMENTS / 2), 2);
        assert_eq!(batch_size(tiny, 0), BATCH_CAP);
    }
}
