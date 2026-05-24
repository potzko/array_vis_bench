pub use crate::utils::shell_sequences::{
    Ciura, Classic, GapSequence, Hibbard, Knuth, Optimized256, Pratt, Sedgewick,
    SedgewickBranching, Tokuda,
};

use crate::traits::log_traits::SortLogger;
use crate::traits::SortFn;

pub struct GapSequenceEntry {
    pub name: &'static str,
    pub big_o: &'static str,
    /// Navigation path for the tree menu, e.g. `["shell sorts", "shell sort", "ciura"]`.
    pub path: &'static [&'static str],
    pub sort_fn: SortFn,
    pub sort_vis: fn(&mut [usize], &mut dyn SortLogger<usize>),
}

#[linkme::distributed_slice]
pub static GAP_SEQUENCES: [GapSequenceEntry] = [..];

// ---------------------------------------------------------------------------
// Inner registration: one module per (sort type, gap sequence) pair.
// ---------------------------------------------------------------------------
macro_rules! register_shell_variant {
    // `$sort_ty` is the concrete `ShellSort::<Seq>` (or `ShellSortOrdered::<Seq>`)
    // type — used to pull all structured fields from the per-axis
    // composable annotation pipeline. The legacy `$big_o` string is kept
    // for the in-house `GapSequenceEntry` slice (display only).
    ($mod:ident, $sort_name:expr, $path:expr, $big_o:expr, $sort_ty:ty, $call:expr) => {
        mod $mod {
            use super::*;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const SORT_NAME: &str = $sort_name;
            const PATH: &[&str] = $path;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { $call(arr, logger) }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { $call(arr, logger) }

            fn run_with_input(
                input_name: &str,
                config: &crate::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                crate::bench_registry::run_sort_with_input(input_name, config, sort_vis, logger);
            }

            fn run_correctness() {
                crate::bench_registry::correctness::sort_battery(sort_fn, SORT_NAME);
                crate::bench_registry::correctness::sort_stability_battery(
                    sort_fn,
                    SORT_NAME,
                    <$sort_ty as crate::traits::composable::HasStability>::STABLE,
                );
            }

            #[linkme::distributed_slice(GAP_SEQUENCES)]
            static ENTRY: GapSequenceEntry = GapSequenceEntry {
                name: SORT_NAME,
                big_o: $big_o,
                path: PATH,
                sort_fn,
                sort_vis,
            };

            #[linkme::distributed_slice(crate::bench_registry::ALGORITHMS)]
            static ALGO_ENTRY: crate::bench_registry::AlgorithmEntry =
                crate::bench_registry::AlgorithmEntry {
                    name: SORT_NAME,
                    category: crate::bench_registry::Category::Sort,
                    worst: <$sort_ty as crate::traits::composable::HasTimeBounds>::WORST,
                    best: <$sort_ty as crate::traits::composable::HasTimeBounds>::BEST,
                    average: <$sort_ty as crate::traits::composable::HasTimeBounds>::AVERAGE,
                    space: <$sort_ty as crate::traits::composable::HasSpace>::SPACE,
                    stable: <$sort_ty as crate::traits::composable::HasStability>::STABLE,
                    adaptive: false,
                    max_input_size: None,
                    run_with_input,
                    run_correctness,
                };

            #[cfg(test)]
            mod sort_test {
                #[test]
                fn correctness() {
                    crate::bench_registry::test_helpers::check_sort_subprocess_assert(&super::ALGO_ENTRY, crate::bench_registry::test_helpers::DEFAULT_TIMEOUT);
                }
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Usage:  register_sequence!(mod_name, mod_name_ordered, SequenceType)
// To add a new gap sequence:
//   1. Add the struct + GapSequence impl to utils/shell_sequences/mod.rs
//   2. Add it to the re-export list at the top of this file
//   3. Call register_sequence!(name, name_ordered, Type) — nothing else changes
// ---------------------------------------------------------------------------
macro_rules! register_sequence {
    ($mod:ident, $mod_ord:ident, $seq:ident) => {
        register_shell_variant!(
            $mod,
            const_format::concatcp!("shell sort<sequence: ", $seq::NAME, ">"),
            &["shell sorts", "shell sort", $seq::NAME],
            $seq::BIG_O,
            crate::sorts::shell_sorts::shell_sort::ShellSort::<$seq>,
            crate::sorts::shell_sorts::shell_sort::ShellSort::<$seq>::sort
        );
        register_shell_variant!(
            $mod_ord,
            const_format::concatcp!("shell sort ordered<sequence: ", $seq::NAME, ">"),
            &["shell sorts", "shell sort ordered", $seq::NAME],
            $seq::BIG_O,
            crate::sorts::shell_sorts::shell_sort_ordered::ShellSortOrdered::<$seq>,
            crate::sorts::shell_sorts::shell_sort_ordered::ShellSortOrdered::<$seq>::sort
        );
    };
}

register_sequence!(classic,              classic_ordered,              Classic);
register_sequence!(knuth,                knuth_ordered,                Knuth);
register_sequence!(hibbard,              hibbard_ordered,              Hibbard);
register_sequence!(sedgewick,            sedgewick_ordered,            Sedgewick);
register_sequence!(sedgewick_branching,  sedgewick_branching_ordered,  SedgewickBranching);
register_sequence!(ciura,                ciura_ordered,                Ciura);
register_sequence!(tokuda,               tokuda_ordered,               Tokuda);
register_sequence!(pratt,                pratt_ordered,                Pratt);
register_sequence!(optimized256,         optimized256_ordered,         Optimized256);
