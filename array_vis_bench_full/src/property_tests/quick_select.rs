//! Property: quick-select places the rank-`target` value at `arr[target]`
//! with `arr[..target] ≤ pivot ≤ arr[target+1..]` and preserves the
//! multiset.

use proptest::prelude::*;

use super::{check, n_cap, MIN_N};
use crate::bench_registry::correctness::QuickSelectFnPtr;
use crate::traits::log_traits::NoOpLogger;

pub fn run(qs_fn: QuickSelectFnPtr, name: &str) {
    let max_n = n_cap(name);
    if max_n < MIN_N {
        return;
    }
    let strategy = (MIN_N..=max_n).prop_flat_map(|n| {
        let max_val = (n * 4).max(8);
        (prop::collection::vec(0usize..max_val, n), 0usize..n)
    });
    check(name, "quick-select", strategy, |(arr, target)| {
        let mut input = arr.clone();
        let mut sorted_reference = arr;
        sorted_reference.sort();
        qs_fn(&mut input, target, &mut NoOpLogger);
        let expected = sorted_reference[target];
        prop_assert_eq!(input[target], expected);
        for &v in &input[..target] {
            prop_assert!(v <= expected, "left pivot violation: {} > {}", v, expected);
        }
        for &v in &input[target + 1..] {
            prop_assert!(v >= expected, "right pivot violation: {} < {}", v, expected);
        }
        let mut sorted_input = input;
        sorted_input.sort();
        prop_assert_eq!(sorted_input, sorted_reference);
        Ok(())
    });
}
