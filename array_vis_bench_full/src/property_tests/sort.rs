//! Property: sort output equals `input.sorted()` (sorted + permutation).

use proptest::prelude::*;

use super::{check, n_cap, vec_strategy, MIN_N};
use crate::bench_registry::correctness::SortFnPtr;
use crate::traits::log_traits::NoOpLogger;

pub fn run(sort_fn: SortFnPtr, name: &str) {
    let max_n = n_cap(name);
    if max_n < MIN_N {
        return;
    }
    check(name, "sort", vec_strategy(max_n), |arr| {
        let mut input = arr.clone();
        let mut expected = arr;
        expected.sort();
        sort_fn(&mut input, &mut NoOpLogger);
        prop_assert_eq!(input, expected);
        Ok(())
    });
}
