//! Random shell sort — shell sort with a randomly-generated gap sequence.
//!
//! Generates `~sqrt(n)` random gap values plus a forced final gap of `1`,
//! sorts the gap sequence in ascending order, then runs shell sort passes
//! in descending gap order. The trailing `1`-gap pass is plain insertion
//! sort, which is what guarantees correctness regardless of the (random)
//! earlier gaps.
//!
//! The gap sequence is itself sorted by **recursively** invoking random
//! shell sort on it (operating on the aux usize array via the `_u`
//! logger family). The recursion bottoms out trivially at size 1 — no
//! insertion-sort fallback — so every sort in the chain is "the same
//! algorithm", and the visualiser sees a nested cascade of aux arrays
//! shrinking as the recursion deepens.
//!
//! Two flavours of [`GapDistribution`] are provided:
//!
//! - [`UniformDist`] — gap values are uniformly drawn from `[0, len)`.
//! - [`ParabolicDist`] — `u ~ Uniform[0,1)`, `gap = (u² · len)`. Skews
//!   the distribution toward small gaps (quadratic density near 0), so
//!   more fine-grained passes and fewer wide jumps.

use std::marker::PhantomData;

use rand::Rng;

use sort_logger::SortLogger;

// ── Gap distribution ─────────────────────────────────────────────────────────
//
// `GapDistribution` trait + concrete impls live in the
// `gap_distribution_lib` leaf crate. Re-exported here so existing paths
// (e.g. `super::random_shell_sort::UniformDist`) keep resolving.
pub use gap_distribution_lib::{
    CubicDist, Distinct, GapDistribution, LogUniformDist, ParabolicDist, UniformDist,
};

// ── RandomShellSort ──────────────────────────────────────────────────────────

/// Minimum ratio between consecutive *executed* gaps. A candidate gap
/// `g` is skipped when `g * MIN_PASS_RATIO > last_executed`, i.e. when
/// the gap didn't shrink by at least a factor of two from the previous
/// pass. `gap = 1` is never skipped by this rule — the only way 1 can
/// trip it is when the previous gap was also 1, which is impossible
/// after deduping and harmless visually otherwise.
const MIN_PASS_RATIO: usize = 2;

#[inline]
fn keep_gap(gap: usize, last_executed: Option<usize>) -> bool {
    if gap == 0 {
        return false;
    }
    match last_executed {
        Some(prev) => gap.saturating_mul(MIN_PASS_RATIO) <= prev,
        None => true,
    }
}

pub struct RandomShellSort<D: GapDistribution>(PhantomData<D>);

impl<D: GapDistribution> RandomShellSort<D> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        let mut rnd = rand::thread_rng();
        let mut gaps = build_gaps::<D, T, U, _>(arr.len(), &mut rnd, logger);
        gap_sort_recursive::<D, T, U, _>(&mut gaps, &mut rnd, logger);

        let mut last_executed: Option<usize> = None;
        for &gap in gaps.iter().rev() {
            if !keep_gap(gap, last_executed) {
                continue;
            }
            for i in gap..arr.len() {
                let temp = arr[i];
                let mut j = i;
                while j >= gap && logger.cmp_gt_data(arr, j - gap, temp) {
                    logger.write(arr, j, j - gap);
                    j -= gap;
                }
                logger.write_data(arr, j, temp);
            }
            last_executed = Some(gap);
        }
        if last_executed != Some(1) {
            // build_gaps always pins a `1` in the sequence, but the
            // ratio filter or duplicate values could leave it unexecuted
            // in degenerate cases. Run the gap=1 pass unconditionally
            // so correctness never depends on the random sequence.
            let gap = 1usize;
            for i in gap..arr.len() {
                let temp = arr[i];
                let mut j = i;
                while j >= gap && logger.cmp_gt_data(arr, j - gap, temp) {
                    logger.write(arr, j, j - gap);
                    j -= gap;
                }
                logger.write_data(arr, j, temp);
            }
        }
        logger.free_aux_arr(&gaps);
    }
}

/// Generate `~sqrt(len)` random gap values into a fresh aux `Vec<usize>`,
/// pinning the final slot to `1` (the correctness anchor). The vector is
/// logged with the visualiser via the `_u` aux family before being
/// returned, so the caller is responsible for the matching
/// `free_aux_arr`.
fn build_gaps<D, T, U, R>(len: usize, rng: &mut R, logger: &mut U) -> Vec<usize>
where
    D: GapDistribution,
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    R: Rng,
{
    let count = ((len as f64).sqrt() as usize).max(1);
    let mut ret = logger.create_aux_arr(count);
    if D::DEDUPE {
        let mut used: std::collections::HashSet<usize> = std::collections::HashSet::new();
        used.insert(1);
        for i in 0..count.saturating_sub(1) {
            let g = sample_distinct::<D, _>(rng, len, &mut used);
            logger.write_data_u(&mut ret, i, g);
        }
    } else {
        for i in 0..count.saturating_sub(1) {
            let g = D::sample(rng, len);
            logger.write_data_u(&mut ret, i, g);
        }
    }
    logger.write_data_u(&mut ret, count - 1, 1);
    ret
}

/// Draw from `D`, rejecting any value already in `used`. After 1024
/// rejected tries (highly unlikely except when the distribution's
/// support is exhausted), fall back to the next unused non-negative
/// integer so the caller always terminates.
fn sample_distinct<D, R>(rng: &mut R, len: usize, used: &mut std::collections::HashSet<usize>) -> usize
where
    D: GapDistribution,
    R: Rng,
{
    for _ in 0..1024 {
        let g = D::sample(rng, len);
        if used.insert(g) {
            return g;
        }
    }
    let mut g = 0usize;
    while !used.insert(g) {
        g += 1;
    }
    g
}

/// Sort the gap array `arr` (itself `Vec<usize>` aux) by **recursively**
/// applying random shell sort. Each call generates its own meta-gap
/// array, recurses on that, and runs `_u`-flavoured shell passes on
/// `arr`. The base case is size 1 — trivially sorted.
fn gap_sort_recursive<D, T, U, R>(arr: &mut [usize], rng: &mut R, logger: &mut U)
where
    D: GapDistribution,
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
    R: Rng,
{
    if arr.len() < 2 {
        return;
    }
    let mut meta_gaps = build_gaps::<D, T, U, _>(arr.len(), rng, logger);
    gap_sort_recursive::<D, T, U, _>(&mut meta_gaps, rng, logger);

    let mut last_executed: Option<usize> = None;
    for &gap in meta_gaps.iter().rev() {
        if !keep_gap(gap, last_executed) {
            continue;
        }
        for i in gap..arr.len() {
            let temp = arr[i];
            let mut j = i;
            while j >= gap && logger.cmp_gt_data_u(arr, j - gap, temp) {
                logger.write_u(arr, j, j - gap);
                j -= gap;
            }
            logger.write_data_u(arr, j, temp);
        }
        last_executed = Some(gap);
    }
    if last_executed != Some(1) {
        let gap = 1usize;
        for i in gap..arr.len() {
            let temp = arr[i];
            let mut j = i;
            while j >= gap && logger.cmp_gt_data_u(arr, j - gap, temp) {
                logger.write_u(arr, j, j - gap);
                j -= gap;
            }
            logger.write_data_u(arr, j, temp);
        }
    }
    logger.free_aux_arr(&meta_gaps);
}

// Random gap sequences pull entropy from `thread_rng()`, so successive runs
// produce different `SortLog` traces even for identical input. Opt the
// leaves out of the determinism check.
#[cfg(feature = "self_register")]
array_vis_bench_core::register_nondeterministic!("random shell sort<uniform>");
#[cfg(feature = "self_register")]
array_vis_bench_core::register_nondeterministic!("random shell sort<parabolic>");
#[cfg(feature = "self_register")]
array_vis_bench_core::register_nondeterministic!("random shell sort<cubic>");
#[cfg(feature = "self_register")]
array_vis_bench_core::register_nondeterministic!("random shell sort<log uniform>");
#[cfg(feature = "self_register")]
array_vis_bench_core::register_nondeterministic!("random shell sort<distinct parabolic>");
