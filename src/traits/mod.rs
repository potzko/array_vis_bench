pub mod log_traits;
pub mod sort_traits;

pub use log_traits::*;
pub use sort_traits::*;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    pub static ref SORT_REGISTRY: Mutex<HashMap<String, Box<dyn Fn(&mut [i32], &mut log_traits::NoOpLogger) + Send + Sync>>> =
        Mutex::new(HashMap::new());
    
    pub static ref SORT_NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

/// Trait for types that can be registered in the global sort registry
pub trait SortRegistry {
    /// Register this sort in the global registry
    fn register();

    /// Sort function that will be called by the registry
    fn sort<T: Ord + Copy, U: log_traits::SortLogger<T>>(arr: &mut [T], logger: &mut U);
}

/// Get all registered sort names
pub fn get_registered_sorts() -> Vec<String> {
    SORT_NAMES.lock().unwrap().clone()
}

/// Get a sort function by name
pub fn get_sort(
    name: &str,
) -> Option<&'static (dyn Fn(&mut [i32], &mut log_traits::NoOpLogger) + Send + Sync)> {
    // This is a bit tricky because we can't easily return a reference to the boxed function
    // For now, we'll return None and implement this differently
    None
}

/// Initialize the sort registry
pub fn init_sort_registry() {
    // This function is called to ensure the registry is initialized
    // The actual initialization happens when sorts are registered
}

/// Register a sort in the global registry
pub fn register_sort(name: &str, _big_o: &str, _stable: bool, _category: &str) {
    // Add the sort name to our list
    let mut sort_names = SORT_NAMES.lock().unwrap();
    if !sort_names.contains(&name.to_string()) {
        sort_names.push(name.to_string());
    }
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
/// ```
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
    };
}
