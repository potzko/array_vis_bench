//! Tiny array generators used by the input registry. Each returns a
//! freshly-allocated `Vec<usize>` so the caller owns the buffer and
//! can sort it in place. These are convenience helpers, not the
//! authoritative input registry — see [`crate::inputs`] for the named
//! input shapes that drive the benchmark/visualiser pipeline.

#![allow(dead_code)]
use rand::Rng;

/// Uniformly random `usize` array. Each element is sampled from the
/// full `usize` range — duplicates expected.
#[must_use]
pub fn get_rand_arr(length: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen()).collect()
}

/// Uniformly random `usize` array clamped to `[min, max)`. Use when
/// the test cares about specific value distributions (e.g. heavy
/// duplicates).
#[must_use]
pub fn get_rand_arr_in_range(length: usize, min: usize, max: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen_range(min..max)).collect()
}

/// `[0, 1, 2, …, length)` — already-sorted ascending. Useful as a
/// best-case input for adaptive sorts.
#[must_use]
pub fn get_arr(length: usize) -> Vec<usize> {
    (0..length).collect()
}

/// `[length-1, length-2, …, 0]` — strictly descending. Worst case for
/// quicksort with first-element pivot and similar.
#[must_use]
pub fn get_reversed_arr(length: usize) -> Vec<usize> {
    (0..length).rev().collect()
}
