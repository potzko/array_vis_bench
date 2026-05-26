//! One-off complexity probe + correctness check for
//! `random shell sort<{D}>`. Runs each variant at several sizes (averaged),
//! prints (n, actions, slope), and asserts the sort actually produces a
//! sorted array.

use fun_sorts_lib::random_shell_sort::{
    CubicDist, Distinct, LogUniformDist, ParabolicDist, RandomShellSort, UniformDist,
};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use sort_logger::{NoOpLogger, SortLog, SortLogger, VisualizerLogger};
use std::marker::PhantomData;

trait CountingSort {
    fn count(arr: &mut [usize]) -> usize;
    fn correctness(arr: &mut [usize]);
}

macro_rules! impl_counting {
    ($name:ident, $ty:ty) => {
        struct $name;
        impl CountingSort for $name {
            fn count(arr: &mut [usize]) -> usize {
                let mut logger = VisualizerLogger::<usize> {
                    log: Vec::<SortLog<usize>>::new(),
                    type_ghost: PhantomData,
                };
                <$ty>::sort(arr, &mut logger as &mut dyn SortLogger<usize>);
                logger.log.len()
            }
            fn correctness(arr: &mut [usize]) {
                <$ty>::sort(arr, &mut NoOpLogger);
            }
        }
    };
}

impl_counting!(Uni, RandomShellSort<UniformDist>);
impl_counting!(Par, RandomShellSort<ParabolicDist>);
impl_counting!(Cub, RandomShellSort<CubicDist>);
impl_counting!(Log, RandomShellSort<LogUniformDist>);
impl_counting!(Dis, RandomShellSort<Distinct<ParabolicDist>>);

fn run_once<C: CountingSort>(n: usize, seed: u64) -> usize {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut arr: Vec<usize> = (0..n).collect();
    arr.shuffle(&mut rng);
    let mut check = arr.clone();
    C::correctness(&mut check);
    assert!(check.windows(2).all(|w| w[0] <= w[1]), "unsorted at n={n}");
    C::count(&mut arr)
}

fn measure<C: CountingSort>(name: &str, sizes: &[usize], trials: u64) {
    println!("\n=== {} ===", name);
    println!("{:>8}  {:>14}  {:>10}  {:>10}", "n", "ops (avg)", "ops/n^1.5", "slope");
    let mut prev: Option<(f64, f64)> = None;
    for &n in sizes {
        let mut total = 0u128;
        for s in 0..trials {
            total += run_once::<C>(n, s) as u128;
        }
        let avg = total as f64 / trials as f64;
        let scaled_15 = avg / (n as f64).powf(1.5);
        let slope = match prev {
            Some((pn, pa)) => (avg.ln() - pa.ln()) / ((n as f64).ln() - pn.ln()),
            None => f64::NAN,
        };
        println!(
            "{:>8}  {:>14.0}  {:>10.4}  {:>10.4}",
            n, avg, scaled_15, slope
        );
        prev = Some((n as f64, avg));
    }
}

fn main() {
    let sizes = [500usize, 1_000, 2_000, 4_000, 8_000, 16_000, 32_000];
    let trials = 5;
    measure::<Uni>("random shell sort<uniform>", &sizes, trials);
    measure::<Par>("random shell sort<parabolic>", &sizes, trials);
    measure::<Cub>("random shell sort<cubic>", &sizes, trials);
    measure::<Log>("random shell sort<log uniform>", &sizes, trials);
    measure::<Dis>("random shell sort<distinct parabolic>", &sizes, trials);
}
