//! Property: after partition, `max(left) ≤ min(right)`, indices don't
//! cross, and the array is a permutation of the input.

use proptest::prelude::*;

use super::{check, n_cap, vec_strategy};
use crate::bench_registry::correctness::PartitionFnPtr;
use crate::traits::log_traits::NoOpLogger;

/// Per-category floor. Pivot selectors like Ninther want `n ≥ 9`;
/// bumping the floor keeps generated inputs in-band for every selector.
const MIN_N: usize = 16;

pub fn run(partition_fn: PartitionFnPtr, name: &str) {
    let max_n = n_cap(name);
    if max_n < MIN_N {
        return;
    }
    let strategy = vec_strategy(max_n)
        .prop_filter("len >= partition floor", move |v| v.len() >= MIN_N);
    check(name, "partition", strategy, |arr| {
        let mut work = arr.clone();
        let mut sorted_original = arr;
        sorted_original.sort();
        let (left_end, right_start) = partition_fn(&mut work, &mut NoOpLogger);
        prop_assert!(
            left_end <= right_start,
            "left_end={left_end} > right_start={right_start}"
        );
        let max_left = work[..left_end].iter().copied().max().unwrap_or(usize::MIN);
        let min_right = work[right_start..].iter().copied().min().unwrap_or(usize::MAX);
        prop_assert!(
            max_left <= min_right,
            "max_left={max_left} > min_right={min_right}"
        );
        let mut sorted_work = work;
        sorted_work.sort();
        prop_assert_eq!(sorted_work, sorted_original);
        Ok(())
    });
}
