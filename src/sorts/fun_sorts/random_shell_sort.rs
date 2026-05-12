//! Random shell sort — shell sort with a randomly-generated gap sequence.
//!
//! Generates `~sqrt(n)` random gap values in `[0, n)` plus a forced final
//! gap of `1`, sorts the gap sequence in ascending order, then runs shell
//! sort passes in descending gap order. The trailing `1`-gap pass is
//! plain insertion sort, which is what guarantees correctness regardless
//! of the (random) earlier gaps.
//!
//! Each call generates a fresh sequence so two runs on identical input
//! produce different intermediate states — the visualiser sees real
//! randomness, and `cargo test` may exercise different gap shapes
//! across invocations. The trailing `1`-gap pass keeps the result
//! deterministic / correct.

use rand::Rng;

use crate::traits::log_traits::SortLogger;

pub struct RandomShellSort;

impl RandomShellSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        if arr.len() < 2 {
            return;
        }
        let jumps = random_jumps(arr.len(), logger);
        for &jump in jumps.iter().rev() {
            // jump==0 would be a no-op pass; skip defensively (random
            // values into the gap array can land at 0 even though we
            // ask for `0..len`).
            if jump == 0 {
                continue;
            }
            for i in jump..arr.len() {
                let temp = arr[i];
                let mut j = i;
                while j >= jump && logger.cmp_gt_data(arr, j - jump, temp) {
                    logger.write(arr, j, j - jump);
                    j -= jump;
                }
                logger.write_data(arr, j, temp);
            }
        }
        logger.free_aux_arr(&jumps);
    }
}

/// Generate the gap sequence: `sqrt(n)` random values in `[0, n)`, with the
/// last slot pinned to `1` (the correctness anchor), then sorted ascending.
fn random_jumps<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    len: usize,
    logger: &mut U,
) -> Vec<usize> {
    let count = ((len as f64).sqrt() as usize).max(1);
    let mut rnd = rand::thread_rng();
    let mut ret = logger.create_aux_arr(count);
    for i in 0..count.saturating_sub(1) {
        logger.write_data_u(&mut ret, i, rnd.gen_range(0..len));
    }
    logger.write_data_u(&mut ret, count - 1, 1);
    sort_jumps(&mut ret, logger);
    ret
}

/// In-place insertion sort over the gap array. Operates on the aux array
/// rather than the main array, so uses the `_u` (usize) logger family.
fn sort_jumps<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [usize], logger: &mut U) {
    for i in 1..arr.len() {
        let temp = arr[i];
        let mut j = i;
        while j > 0 && logger.cmp_gt_data_u(arr, j - 1, temp) {
            logger.write_u(arr, j, j - 1);
            j -= 1;
        }
        logger.write_data_u(arr, j, temp);
    }
}

sort_registry_macro::sort_family! {
    type Sort = RandomShellSort;
    name        = "random shell sort";
    big_o       = "O(N^2.5)";
    stable      = false;
    direct_sort = true;
    path        = ["fun sorts", "random shell sort"];
    max_n_for_tests = 1000;
}
