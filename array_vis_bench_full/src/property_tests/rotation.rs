//! Property: rotating `(0..n)` at index `split` yields
//! `[split..n] ++ [0..split]`.

use proptest::prelude::*;

use super::{check, n_cap, MIN_N};
use crate::bench_registry::correctness::RotationFnPtr;
use crate::traits::log_traits::NoOpLogger;

pub fn run(rotate_fn: RotationFnPtr, name: &str) {
    let max_n = n_cap(name);
    if max_n < MIN_N {
        return;
    }
    let strategy = (MIN_N..=max_n).prop_flat_map(|n| (Just(n), 0usize..=n));
    check(name, "rotation", strategy, |(n, split)| {
        let original: Vec<usize> = (0..n).collect();
        let mut arr = original.clone();
        rotate_fn(&mut arr, split, &mut NoOpLogger);
        let mut expected: Vec<usize> = original[split..].to_vec();
        expected.extend_from_slice(&original[..split]);
        prop_assert_eq!(arr, expected);
        Ok(())
    });
}
