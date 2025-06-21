pub mod log_traits;
pub mod sort_traits;

pub use log_traits::*;
pub use sort_traits::*;

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
