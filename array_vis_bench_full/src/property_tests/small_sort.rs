//! Property: small-sort produces a sorted permutation up to its declared
//! threshold (its contract is "len ≤ THRESHOLD").

use proptest::prelude::*;

use super::{check, MIN_N};
use crate::bench_registry::correctness::SmallSortFnPtr;
use crate::traits::log_traits::NoOpLogger;

pub fn run(sort_fn: SmallSortFnPtr, name: &str, threshold: usize) {
    if threshold < MIN_N {
        return;
    }
    let strategy = (MIN_N..=threshold)
        .prop_flat_map(|n| prop::collection::vec(0usize..(n * 4).max(8), n));
    check(name, "small-sort", strategy, |arr| {
        let mut input = arr.clone();
        let mut expected = arr;
        expected.sort();
        sort_fn(&mut input, &mut NoOpLogger);
        prop_assert_eq!(input, expected);
        Ok(())
    });
}
