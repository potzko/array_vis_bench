use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use sort_logger::SortLogger;

pub struct CycleSort;

// Composable annotations — the spec compiler inherits each entry's complexity
// from the concrete type. Cycle sort does a full counting pass per element in
// every case, so it is Θ(N²) regardless of input; in-place; not stable (writes
// displaced elements out of order).
impl HasTimeBounds for CycleSort {
    const WORST: Complexity = Complexity::N_SQUARED;
    const BEST: Complexity = Complexity::N_SQUARED;
    const AVERAGE: Complexity = Complexity::N_SQUARED;
}

impl HasSpace for CycleSort {
    const SPACE: Complexity = Complexity::CONST;
}

impl HasStability for CycleSort {
    const STABLE: bool = false;
}

impl CycleSort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        let n = arr.len();
        if n < 2 {
            return;
        }
        for cycle_start in 0..n - 1 {
            let mut item = arr[cycle_start];
            let mut pos = cycle_start;
            for i in cycle_start + 1..n {
                if logger.cmp_lt_data(arr, i, item) {
                    pos += 1;
                }
            }
            if pos == cycle_start {
                continue;
            }
            while item == arr[pos] {
                pos += 1;
            }
            if pos != cycle_start {
                let displaced = arr[pos];
                logger.write_data(arr, pos, item);
                item = displaced;
            }
            while pos != cycle_start {
                pos = cycle_start;
                for i in cycle_start + 1..n {
                    if logger.cmp_lt_data(arr, i, item) {
                        pos += 1;
                    }
                }
                while item == arr[pos] {
                    pos += 1;
                }
                let displaced = arr[pos];
                logger.write_data(arr, pos, item);
                item = displaced;
            }
        }
    }
}

// Legacy self-registration — gated OFF by default; the spec compiler
// (`spec_catalog`) is the canonical registrar. See `[features]` in Cargo.toml.
#[cfg(feature = "self_register")]
sort_registry_macro::sort_family! {
    type Sort = CycleSort;
    name        = "cycle sort";
    big_o       = "O(N^2)";
    stable      = false;
    direct_sort = true;
    path        = ["cycle sorts", "cycle sort"];
}
