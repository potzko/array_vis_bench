// Pure sequence implementations live in utils::shell_sequences.
// This module owns the registration infrastructure and re-exports the
// sequence types so the rest of shell_sorts can import from one place.
pub use crate::utils::shell_sequences::{
    Ciura, Classic, GapSequence, Hibbard, Knuth, Optimized256, Pratt, Sedgewick,
    SedgewickBranching, Tokuda,
};

use crate::traits::log_traits::SortLogger;
use crate::traits::SortFn;

/// A runtime entry describing a concrete shell-sort variant.
///
/// Populated at link time via `#[linkme::distributed_slice(GAP_SEQUENCES)]`
/// entries in each sequence's registration block below.  `combinations.rs`
/// iterates this slice at startup to fill SORT_REGISTRY / SORT_NAMES.
/// `shell_sorts::fn_sort` iterates it for visualization dispatch.
pub struct GapSequenceEntry {
    pub name: &'static str,
    pub big_o: &'static str,
    /// Monomorphic sort for SORT_REGISTRY (NoOpLogger, fully inlinable).
    pub sort_fn: SortFn,
    /// Sort with dynamic logger dispatch, used by fn_sort for visualization.
    pub sort_vis: fn(&mut [usize], &mut dyn SortLogger<usize>),
}

#[linkme::distributed_slice]
pub static GAP_SEQUENCES: [GapSequenceEntry] = [..];

// ---------------------------------------------------------------------------
// Registration macro
//
// Generates two private submodules — one for the standard shell sort and one
// for the ordered-insertion variant — plus four distributed-slice statics for
// a given GapSequence type.
//
// Usage:  register_sequence!(mod_name, mod_name_ordered, SequenceType)
//
// To add a new gap sequence:
//   1. Add the struct + GapSequence impl to utils/shell_sequences/mod.rs
//   2. Add it to the re-export list at the top of this file
//   3. Call register_sequence!(name, name_ordered, Type) below — nothing else changes
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
