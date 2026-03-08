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
// Usage:  register_sequence!(mod_name, mod_name_ordered, SequenceType)
// To add a new gap sequence:
//   1. Add the struct + GapSequence impl to utils/shell_sequences/mod.rs
//   2. Add it to the re-export list at the top of this file
//   3. Call register_sequence!(name, name_ordered, Type) — nothing else changes
// ---------------------------------------------------------------------------
macro_rules! register_sequence {
    ($mod:ident, $mod_ord:ident, $seq:ident) => {
        mod $mod {
            use crate::sorts::shell_sorts::sequences::{GAP_SEQUENCES, GapSequence, GapSequenceEntry};
            use crate::sorts::shell_sorts::shell_sort::ShellSort;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            use super::$seq;

            const SORT_NAME: &str =
                const_format::concatcp!("shell sort<sequence: ", $seq::NAME, ">");
            const PATH: &[&str] = &["shell sorts", "shell sort", $seq::NAME];

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) {
                ShellSort::<$seq>::sort(arr, logger);
            }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
                ShellSort::<$seq>::sort(arr, logger);
            }
            fn bench(arr: &mut [usize]) {
                let mut l = NoOpLogger;
                ShellSort::<$seq>::sort(arr, &mut l);
            }

            #[linkme::distributed_slice(GAP_SEQUENCES)]
            static ENTRY: GapSequenceEntry = GapSequenceEntry {
                name: SORT_NAME,
                big_o: $seq::BIG_O,
                path: PATH,
                sort_fn,
                sort_vis,
            };

            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry {
                    name: SORT_NAME,
                    big_o: $seq::BIG_O,
                    stable: false,
                    run: bench,
                };
        }

        mod $mod_ord {
            use crate::sorts::shell_sorts::sequences::{GAP_SEQUENCES, GapSequence, GapSequenceEntry};
            use crate::sorts::shell_sorts::shell_sort_ordered::ShellSortOrdered;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};
            use super::$seq;

            const SORT_NAME: &str =
                const_format::concatcp!("shell sort ordered<sequence: ", $seq::NAME, ">");
            const PATH: &[&str] = &["shell sorts", "shell sort ordered", $seq::NAME];

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) {
                ShellSortOrdered::<$seq>::sort(arr, logger);
            }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
                ShellSortOrdered::<$seq>::sort(arr, logger);
            }
            fn bench(arr: &mut [usize]) {
                let mut l = NoOpLogger;
                ShellSortOrdered::<$seq>::sort(arr, &mut l);
            }

            #[linkme::distributed_slice(GAP_SEQUENCES)]
            static ENTRY: GapSequenceEntry = GapSequenceEntry {
                name: SORT_NAME,
                big_o: $seq::BIG_O,
                path: PATH,
                sort_fn,
                sort_vis,
            };

            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry {
                    name: SORT_NAME,
                    big_o: $seq::BIG_O,
                    stable: false,
                    run: bench,
                };
        }
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
