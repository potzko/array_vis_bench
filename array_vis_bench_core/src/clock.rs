//! Timing an entry's run with the input built outside the clock. `batch_size`
//! says how many runs to time back to back so a tiny run is not lost in the
//! timer's resolution, and `time_runs` builds that many inputs, times the
//! runs and returns the time of one. The benchmark and the sweep both time
//! through here.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::{Duration, Instant};

use sort_logger::NoOpLogger;

use crate::bench_registry::{find_input, AlgorithmEntry, RunConfig};

/// A batch of runs is at least this long, well past the clock's resolution.
pub const BATCH_FLOOR: Duration = Duration::from_micros(50);

/// The most runs one batch times, so a batch of microsecond runs still ends
/// within the floor's order of magnitude.
pub const BATCH_CAP: usize = 64;

/// How many runs to time back to back when one takes `one`: enough to reach
/// the floor, at least one and at most the cap.
pub fn batch_size(one: Duration) -> usize {
    if one.is_zero() { return BATCH_CAP; }
    let runs = (BATCH_FLOOR.as_nanos() + one.as_nanos() - 1) / one.as_nanos();
    runs.clamp(1, BATCH_CAP as u128) as usize
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
        assert_eq!(batch_size(Duration::from_micros(100)), 1);
        assert_eq!(batch_size(Duration::from_micros(50)), 1);
        assert_eq!(batch_size(Duration::from_micros(20)), 3);
        assert_eq!(batch_size(Duration::from_nanos(1000)), 50);
        assert_eq!(batch_size(Duration::from_nanos(100)), BATCH_CAP);
        assert_eq!(batch_size(Duration::ZERO), BATCH_CAP);
    }
}
