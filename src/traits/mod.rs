pub mod log_traits;
pub mod sort_traits;

pub use log_traits::*;
pub use sort_traits::*;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

/// Function pointer type for sort implementations (fully optimizable, no trait objects)
pub type SortFn = fn(&mut [usize], &mut log_traits::NoOpLogger);

/// Function pointer type for sort visualisation (accepts dyn SortLogger)
pub type SortVisFn = fn(&mut [usize], &mut dyn log_traits::SortLogger<usize>);

lazy_static! {
    pub static ref SORT_REGISTRY: Mutex<HashMap<String, SortFn>> =
        Mutex::new(HashMap::new());

    /// Registry for visualisation dispatch — maps sort name → sort_vis fn pointer.
    /// Populated by `sort_family!(... direct_sort = true; ...)` and by ctors that
    /// iterate family-specific slices (e.g. MERGE_SORTS for rotation variants).
    pub static ref SORT_VIS_REGISTRY: Mutex<HashMap<String, SortVisFn>> =
        Mutex::new(HashMap::new());
}

/// Trait for types that can be registered (metadata side)
pub use sort_registry_core::SortRegistry;

/// Get all registered sort names (from core)
pub fn get_registered_sorts() -> Vec<String> {
    sort_registry_core::get_registered_sorts()
}

/// Build the full navigation tree for the interactive sort-selection menu.
pub fn get_sort_tree() -> sort_registry_core::SortTree {
    sort_registry_core::get_sort_tree()
}

/// Get a sort function by name - returns a bare function pointer (fully inlinable)
pub fn get_sort(name: &str) -> Option<SortFn> {
    SORT_REGISTRY
        .lock()
        .unwrap()
        .get(name)
        .copied()
}

/// Initialize the sort registry (no-op)
pub fn init_sort_registry() {}

/// Register sort metadata (delegates to core)
pub fn register_sort(name: &str, big_o: &str, stable: bool, category: &str) {
    sort_registry_core::register_sort(name, big_o, stable, category)
}

/// Macro to create a sort implementation with reduced boilerplate
///
/// Usage: create_sort!(
///     sort_function_name,
///     "sort name",
///     "big O time complexity",
///     stable_sort
/// )
///
/// Example:
/// ```ignore
/// create_sort!(
///     bubble_sort,
///     "bubble sort",
///     "O(N^2)",
///     true
/// );
/// ```
#[macro_export]
macro_rules! create_sort {
    ($sort_fn:ident, $name:expr, $big_o:expr, $stable:expr) => {
        const BIG_O: &str = $big_o;
        const NAME: &str = $name;
        const STABLE: bool = $stable;

        use crate::traits;
        use std::marker::PhantomData;

        pub struct SortImp<T: Ord + Copy, U: traits::log_traits::SortLogger<T>> {
            _markers: (PhantomData<T>, PhantomData<U>),
        }

        impl<T: Ord + Copy, U: traits::log_traits::SortLogger<T>>
            traits::sort_traits::SortAlgo<T, U> for SortImp<T, U>
        {
            fn big_o() -> &'static str {
                BIG_O
            }
            fn sort(arr: &mut [T], logger: &mut U) {
                $sort_fn::<T, U>(arr, logger);
            }
            fn name() -> &'static str {
                NAME
            }
            fn stable() -> bool {
                STABLE
            }
        }

        // Monomorphic registration type for derive-based registry
        #[derive(sort_registry_macro::SortRegistry)]
        pub struct SortReg;

        impl traits::sort_traits::SortAlgo<usize, traits::log_traits::NoOpLogger> for SortReg {
            fn big_o() -> &'static str { BIG_O }
            fn name() -> &'static str { NAME }
            fn sort(arr: &mut [usize], logger: &mut traits::log_traits::NoOpLogger) {
                $sort_fn::<usize, traits::log_traits::NoOpLogger>(arr, logger);
            }
            fn stable() -> bool { STABLE }
        }

        // Bench-time static registration (no trait objects) via distributed slice
        // Provides a monomorphic function pointer for benchmarks
        #[allow(non_upper_case_globals)]
        fn __bench_run(arr: &mut [usize]) {
            let mut logger = traits::log_traits::NoOpLogger;
            $sort_fn::<usize, traits::log_traits::NoOpLogger>(arr, &mut logger);
        }

        #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
        static __BENCH_SORT_ENTRY: crate::bench_registry::SortBenchEntry = crate::bench_registry::SortBenchEntry {
            name: NAME,
            big_o: BIG_O,
            stable: STABLE,
            run: __bench_run,
        };

        #[cfg(test)]
        mod sort_test {
            #[test]
            fn correctness() {
                crate::bench_registry::test_helpers::check_sort(&super::__BENCH_SORT_ENTRY);
            }
        }
    };
}
