//! Property: merging two pre-sorted halves at `mid` yields a sorted array.

use proptest::prelude::*;

use super::{check, n_cap, MIN_N};
use crate::bench_registry::correctness::MergeFnPtr;
use crate::traits::log_traits::NoOpLogger;

pub fn run(merge_fn: MergeFnPtr, name: &str) {
    let max_n = n_cap(name);
    if max_n < MIN_N {
        return;
    }
    // Yields `(arr, mid)` where `arr[..mid]` and `arr[mid..]` are each
    // sorted but not necessarily in order across the boundary.
    let strategy = (MIN_N..=max_n).prop_flat_map(|n| {
        let max_val = (n * 4).max(8);
        (0usize..=n, prop::collection::vec(0usize..max_val, n)).prop_map(|(mid, vals)| {
            let mut left: Vec<usize> = vals[..mid].to_vec();
            let mut right: Vec<usize> = vals[mid..].to_vec();
            left.sort();
            right.sort();
            left.extend_from_slice(&right);
            (left, mid)
        })
    });
    check(name, "merge", strategy, |(arr, mid)| {
        let mut input = arr.clone();
        let mut expected = arr;
        expected.sort();
        merge_fn(&mut input, mid, &mut NoOpLogger);
        prop_assert_eq!(input, expected);
        Ok(())
    });
}
