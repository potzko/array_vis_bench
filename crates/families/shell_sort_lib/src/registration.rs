//! Per-(algorithm × sequence) registration for shell sort.
//!
//! Each `register_sequence!` invocation creates two private modules (one
//! for `ShellSort`, one for `ShellSortOrdered`) carrying:
//!   - a `static ENTRY: GapSequenceEntry` in the local `GAP_SEQUENCES`
//!     slice — used by the [`register_shell_sorts`] ctor to populate
//!     `sort_registry_core`'s navigation tree.
//!   - a `static ALGO_ENTRY: AlgorithmEntry` in
//!     `array_vis_bench_core::bench_registry::ALGORITHMS` — the per-leaf
//!     registration that lets the wiring crate's harness discover the
//!     variant at link time.

// Only the gated registration modules reference the concrete sequences.
#[cfg(feature = "self_register")]
use crate::sequences::{
    Ciura, Classic, GapSequence, Hibbard, Knuth, Optimized256, Pratt, Sedgewick,
    SedgewickBranching, Tokuda,
};

use sort_logger::SortLogger;

/// Natural-signature sort entry — `NoOpLogger` flavour for the test path.
pub type SortFn = fn(&mut [usize], &mut sort_logger::NoOpLogger);

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
        // Gated: only compiled (and thus only registered) when the crate's
        // `self_register` feature is on. Off → the crate provides types only.
        #[cfg(feature = "self_register")]
        mod $mod {
            use super::*;
            use sort_logger::{NoOpLogger, SortLogger};

            const SORT_NAME: &str = $sort_name;
            const PATH: &[&str] = $path;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { $call(arr, logger) }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { $call(arr, logger) }

            fn run_with_input(
                input_name: &str,
                config: &array_vis_bench_core::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                array_vis_bench_core::bench_registry::run_sort_with_input(
                    input_name, config, sort_vis, logger,
                );
            }

            fn run_correctness() {
                array_vis_bench_core::bench_registry::correctness::sort_battery(sort_fn, SORT_NAME);
                array_vis_bench_core::bench_registry::correctness::sort_stability_battery(
                    sort_fn,
                    SORT_NAME,
                    <$sort_ty as array_vis_bench_traits::composable::HasStability>::STABLE,
                );
            }

            #[linkme::distributed_slice(super::GAP_SEQUENCES)]
            static ENTRY: super::GapSequenceEntry = super::GapSequenceEntry {
                name: SORT_NAME,
                big_o: $big_o,
                path: PATH,
                sort_fn,
                sort_vis,
            };

            #[linkme::distributed_slice(array_vis_bench_core::bench_registry::ALGORITHMS)]
            static ALGO_ENTRY: array_vis_bench_core::bench_registry::AlgorithmEntry =
                array_vis_bench_core::bench_registry::AlgorithmEntry {
                    name: SORT_NAME,
                    category: array_vis_bench_core::bench_registry::Category::Sort,
                    worst:   <$sort_ty as array_vis_bench_traits::composable::HasTimeBounds>::WORST,
                    best:    <$sort_ty as array_vis_bench_traits::composable::HasTimeBounds>::BEST,
                    average: <$sort_ty as array_vis_bench_traits::composable::HasTimeBounds>::AVERAGE,
                    space:   <$sort_ty as array_vis_bench_traits::composable::HasSpace>::SPACE,
                    stable:  <$sort_ty as array_vis_bench_traits::composable::HasStability>::STABLE,
                    adaptive: false,
                    max_input_size: None,
                    run_with_input,
                    run_correctness,
                };
        }
    };
}

// ---------------------------------------------------------------------------
// Usage:  register_sequence!(mod_name, mod_name_ordered, SequenceType)
// To add a new gap sequence:
//   1. Add the struct + GapSequence impl to sequences.rs
//   2. Add it to the `use crate::sequences::{…}` import above
//   3. Call register_sequence!(name, name_ordered, Type) — nothing else changes
// ---------------------------------------------------------------------------
macro_rules! register_sequence {
    ($mod:ident, $mod_ord:ident, $seq:ident) => {
        register_shell_variant!(
            $mod,
            const_format::concatcp!("shell sort<sequence: ", $seq::NAME, ">"),
            &["shell sorts", "shell sort", $seq::NAME],
            $seq::BIG_O,
            crate::shell_sort::ShellSort::<$seq>,
            crate::shell_sort::ShellSort::<$seq>::sort
        );
        register_shell_variant!(
            $mod_ord,
            const_format::concatcp!("shell sort ordered<sequence: ", $seq::NAME, ">"),
            &["shell sorts", "shell sort ordered", $seq::NAME],
            $seq::BIG_O,
            crate::shell_sort_ordered::ShellSortOrdered::<$seq>,
            crate::shell_sort_ordered::ShellSortOrdered::<$seq>::sort
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

/// Iterates the GAP_SEQUENCES distributed slice at startup and registers
/// every variant into `sort_registry_core` so the interactive picker's
/// menu tree includes the shell-sort branches in TOML declaration order.
#[cfg(feature = "self_register")]
#[ctor::ctor]
fn register_shell_sorts() {
    for entry in GAP_SEQUENCES {
        let full: Vec<&str> = std::iter::once("sorts").chain(entry.path.iter().copied()).collect();
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, &full);
    }
}
