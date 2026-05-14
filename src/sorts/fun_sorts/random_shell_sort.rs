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

use crate::traits::log_traits::SortLogger;

// ── Gap distribution ─────────────────────────────────────────────────────────

/// One random gap value in `[0, len)`. Each [`GapDistribution`] impl
/// shapes the gap density curve differently.
pub trait GapDistribution {
    fn sample<R: Rng>(rng: &mut R, len: usize) -> usize;
}

/// Uniform distribution: `gap ~ Uniform[0, len)`.
pub struct UniformDist;
combo_codegen::component!(GapDistribution, UniformDist, "uniform");

impl GapDistribution for UniformDist {
    #[inline]
    fn sample<R: Rng>(rng: &mut R, len: usize) -> usize {
        rng.gen_range(0..len)
    }
}

/// Parabolic distribution: `u ~ Uniform[0,1)`, `gap = floor(u² · len)`.
/// Density is quadratic near 0 — gap values cluster at the small end.
pub struct ParabolicDist;
combo_codegen::component!(GapDistribution, ParabolicDist, "parabolic");

impl GapDistribution for ParabolicDist {
    #[inline]
    fn sample<R: Rng>(rng: &mut R, len: usize) -> usize {
        let u: f64 = rng.gen();
        ((u * u) * len as f64) as usize
    }
}

// ── RandomShellSort ──────────────────────────────────────────────────────────

pub struct RandomShellSort<D: GapDistribution>(PhantomData<D>);

impl<D: GapDistribution> RandomShellSort<D> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        let mut rnd = rand::thread_rng();
        let mut gaps = build_gaps::<D, T, U, _>(arr.len(), &mut rnd, logger);
        gap_sort_recursive::<D, T, U, _>(&mut gaps, &mut rnd, logger);

        for &gap in gaps.iter().rev() {
            if gap == 0 {
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
    for i in 0..count.saturating_sub(1) {
        let g = D::sample(rng, len);
        logger.write_data_u(&mut ret, i, g);
    }
    logger.write_data_u(&mut ret, count - 1, 1);
    ret
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

    for &gap in meta_gaps.iter().rev() {
        if gap == 0 {
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
    }
    logger.free_aux_arr(&meta_gaps);
}

combo_codegen::family!(
    type = RandomShellSort<{D}>,
    uses = [
        "crate::sorts::fun_sorts::random_shell_sort::{RandomShellSort, UniformDist, ParabolicDist}",
    ],
    D: GapDistribution,
    name = "random shell sort",
    big_o = "O(N^2.5)",
    stable = false,
    direct_sort = true,
    path = ["fun sorts", "random shell sort", "{D}"],
    max_n_for_tests = 1000,
);
