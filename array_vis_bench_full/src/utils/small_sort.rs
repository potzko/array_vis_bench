//! Small-sort wiring: re-exports from per-kind leaf crates plus the
//! standalone-algorithm registrations.
//!
//! Trait definitions (`SmallSort`, `NonTrivialSmallSort`,
//! `InsertionStrategy`, `SetSizeSmallSort`, `SmallSortAdapter`,
//! `DeferredSmallSort`) live in `array_vis_bench_traits::role::small_sort`.
//! Concrete types live in 6 leaf crates:
//!
//! - `small_sort_insertion_strategy` — LinearInsertion, BinaryInsertion
//! - `small_sort_basic` — NoSmallSort, Size1SmallSort, Size2SmallSort
//! - `small_sort_insertion` — InsertionSmallSort<S, N>
//! - `small_sort_network` — NetworkSmallSort + `sort_network_8`
//! - `small_sort_network_16` — Network16SmallSort
//! - `small_sort_deferred_insertion` — DeferredInsertion<S, N>
//!
//! The `register_small_sort!` macro stays here because it touches a lot
//! of array_vis_bench-internal items (`ALGORITHMS`,
//! `sort_registry_core`, `bench_registry::run_small_sort_with_input`,
//! …). Each registered variant gets one `register_small_sort!` call
//! at the bottom of this file.

pub use array_vis_bench_traits::{
    insertion_sort_with, DeferredSmallSort, InsertionStrategy, NonTrivialSmallSort,
    SetSizeSmallSort, SmallSort, SmallSortAdapter,
};

pub use small_sort_basic::{NoSmallSort, Size1SmallSort, Size2SmallSort};
pub use small_sort_deferred_insertion::DeferredInsertion;
pub use small_sort_insertion::InsertionSmallSort;
pub use small_sort_insertion_strategy::{BinaryInsertion, LinearInsertion};
pub use small_sort_network::NetworkSmallSort;
pub use small_sort_network_16::Network16SmallSort;

use crate::traits::log_traits::SortLogger;

/// Linear insertion sort over the whole array. Kept as a free function
/// because several call sites (circle sorts, etc.) want it without
/// committing to a strategy parameter.
#[inline(always)]
pub(crate) fn insertion_sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
    arr: &mut [T],
    logger: &mut U,
) -> bool {
    insertion_sort_with::<LinearInsertion, _, _>(arr, logger)
}

/// Register a `SmallSort` impl as a standalone algorithm. Sentinel
/// small-sorts (those with `THRESHOLD = 0`) should NOT call this —
/// they're glue, not algorithms.
macro_rules! register_small_sort {
    ($mod:ident, $ty:ty, $variant_name:expr) => {
        mod $mod {
            use super::*;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const NAME: &str = const_format::concatcp!("small-sort: ", $variant_name);

            fn sort_dyn(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
                let _ = <$ty as crate::utils::small_sort::SmallSort>::sort(arr, logger);
            }
            fn sort_noop(arr: &mut [usize], logger: &mut NoOpLogger) {
                let _ = <$ty as crate::utils::small_sort::SmallSort>::sort(arr, logger);
            }

            fn run_with_input(
                input_name: &str,
                config: &crate::bench_registry::RunConfig,
                logger: &mut dyn SortLogger<usize>,
            ) {
                // Clamp input size to the small-sort's declared
                // threshold; behaviour above-threshold is undefined per
                // the trait's contract.
                let threshold = <$ty as crate::utils::small_sort::SmallSort>::THRESHOLD;
                let clamped = crate::bench_registry::RunConfig {
                    size: config.size.min(threshold),
                    seed: config.seed,
                };
                crate::bench_registry::run_small_sort_with_input(
                    input_name, &clamped, sort_dyn, logger,
                );
            }

            fn run_correctness() {
                crate::bench_registry::correctness::small_sort_battery(
                    sort_noop,
                    NAME,
                    <$ty as crate::utils::small_sort::SmallSort>::THRESHOLD,
                );
            }

            // Small-sorts are bounded by THRESHOLD (compile-time const),
            // so their per-invocation time and space are O(1) regardless
            // of the algorithm's intrinsic complexity. The intrinsic
            // `HasStability` impl still drives `stable`.
            #[linkme::distributed_slice(crate::bench_registry::ALGORITHMS)]
            pub(super) static ENTRY: crate::bench_registry::AlgorithmEntry =
                crate::bench_registry::AlgorithmEntry {
                    name: NAME,
                    category: crate::bench_registry::Category::SmallSort,
                    worst: crate::traits::complexity::Complexity::CONST,
                    best: crate::traits::complexity::Complexity::CONST,
                    average: crate::traits::complexity::Complexity::CONST,
                    space: crate::traits::complexity::Complexity::CONST,
                    stable: <$ty as crate::traits::composable::HasStability>::STABLE,
                    adaptive: false,
                    max_input_size: Some(
                        <$ty as crate::utils::small_sort::SmallSort>::THRESHOLD,
                    ),
                    run_with_input,
                    run_correctness,
                };

            #[ctor::ctor]
            fn register_path() {
                sort_registry_core::register_sort_path(
                    NAME,
                    "O(K)",
                    false,
                    &["small-sorts", $variant_name],
                );
            }

            #[cfg(test)]
            mod small_sort_test {
                #[test]
                fn correctness() {
                    crate::bench_registry::test_helpers::check_sort_subprocess_assert(
                        &super::ENTRY,
                        crate::bench_registry::test_helpers::DEFAULT_TIMEOUT,
                    );
                }
            }
        }
    };
}

// Standalone-algorithm registrations.
register_small_sort!(register_size2,         Size2SmallSort, "size: 2");
register_small_sort!(register_ins_linear_16, InsertionSmallSort<LinearInsertion, 16>, "insertion: 16");
register_small_sort!(register_ins_linear_32, InsertionSmallSort<LinearInsertion, 32>, "insertion: 32");
register_small_sort!(register_ins_binary_16, InsertionSmallSort<BinaryInsertion, 16>, "binary insertion: 16");
register_small_sort!(register_ins_binary_32, InsertionSmallSort<BinaryInsertion, 32>, "binary insertion: 32");
register_small_sort!(register_network_8,     NetworkSmallSort,   "network: 8");
register_small_sort!(register_network_16,    Network16SmallSort, "network: 16");
