//! Circle sort variant registration.
//!
//! The circle sort matrix has two dimensions:
//!
//! 1. **Algorithm family** (see sub-module docs for abstraction details):
//!    - Recursive — abstracted over *ordering* (`orderings.rs`)
//!    - Bottom-up — abstracted over *traversal direction* (`directions.rs`)
//!
//! 2. **Finishing strategy** (`finishing.rs`):
//!    - [`Convergence`] — run to full convergence (baseline).
//!    - [`ShortCircuit<S>`] — stop after ⌊log₂(n)⌋ passes and finish with
//!      `S` (a [`NearSortedSort`]).  Currently only `InsertionNearSort`.
//!
//! To add a new ordering or direction: implement the trait in the appropriate
//! file and call the matching macro below.
//! To add a new finishing sort: implement `NearSortedSort` in `finishing.rs`
//! and add a new set of `_sc!` invocations below.

use crate::traits::log_traits::SortLogger;
use crate::traits::SortFn;

pub use super::directions::{Decreasing, Increasing, ShakerDecInc, ShakerIncDec};
pub use super::finishing::{Convergence, InsertionNearSort, ShortCircuit};
pub use super::orderings::{LeftMidRight, PostOrder, PreOrder, RightMidLeft};

pub struct CircleEntry {
    pub name: &'static str,
    pub big_o: &'static str,
    /// Navigation path for the tree menu.
    pub path: &'static [&'static str],
    pub sort_fn: SortFn,
    pub sort_vis: fn(&mut [usize], &mut dyn SortLogger<usize>),
}

#[linkme::distributed_slice]
pub static CIRCLE_ENTRIES: [CircleEntry] = [..];

// ---------------------------------------------------------------------------
// Unified registration macro
//
// Usage:
//   register_circle!(mod_name, "sort name", &["path", ..."], driver_fn::<TypeA, TypeB>);
//
// The $call expression must be a function accepting (&mut [T], &mut U)
// where U: SortLogger<T>.
// ---------------------------------------------------------------------------
macro_rules! register_circle {
    ($mod:ident, $sort_name:expr, $path:expr, $call:expr) => {
        mod $mod {
            use super::*;
            #[allow(unused_imports)]
            use crate::sorts::circle_sorts::finishing::*;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const SORT_NAME: &str = $sort_name;
            const PATH: &[&str] = $path;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) { $call(arr, logger) }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { $call(arr, logger) }
            fn bench(arr: &mut [usize]) { $call(arr, &mut NoOpLogger) }

            #[linkme::distributed_slice(CIRCLE_ENTRIES)]
            static ENTRY: CircleEntry = CircleEntry {
                name: SORT_NAME,
                big_o: "O(N log\u{00B2} N)",
                path: PATH,
                sort_fn,
                sort_vis,
            };

            #[linkme::distributed_slice(crate::bench_registry::BENCH_SORTS)]
            static BENCH_ENTRY: crate::bench_registry::SortBenchEntry =
                crate::bench_registry::SortBenchEntry {
                    name: SORT_NAME,
                    big_o: "O(N log\u{00B2} N)",
                    stable: false,
                    run: bench,
                };

            #[cfg(test)]
            mod sort_test {
                #[test]
                fn correctness() {
                    crate::bench_registry::test_helpers::check_sort_subprocess_assert(&super::BENCH_ENTRY, crate::bench_registry::test_helpers::DEFAULT_TIMEOUT);
                }
            }
        }
    };
}

// Shaker recursive: non-generic, registered inline.
register_circle!(
    shaker_recursive,
    "circle sort (recursive shaker)",
    &["circle sorts", "recursive", "shaker"],
    crate::sorts::circle_sorts::circle_sort_shaker_recursive::CircleSortShakerRecursive::sort
);

// ---------------------------------------------------------------------------
// Recursive — convergence (run to completion)
// ---------------------------------------------------------------------------

register_circle!(
    pre_order, "circle sort (recursive pre-order)",
    &["circle sorts", "recursive", "pre-order"],
    drive_recursive::<PreOrder, Convergence>
);
register_circle!(
    left_mid_right, "circle sort (recursive left-mid-right)",
    &["circle sorts", "recursive", "left-mid-right"],
    drive_recursive::<LeftMidRight, Convergence>
);
register_circle!(
    right_mid_left, "circle sort (recursive right-mid-left)",
    &["circle sorts", "recursive", "right-mid-left"],
    drive_recursive::<RightMidLeft, Convergence>
);
register_circle!(
    post_order, "circle sort (recursive post-order)",
    &["circle sorts", "recursive", "post-order"],
    drive_recursive::<PostOrder, Convergence>
);

// ---------------------------------------------------------------------------
// Recursive — short circuit (stop at log2(n), finish with S)
// ---------------------------------------------------------------------------

register_circle!(
    sc_pre_order,
    const_format::concatcp!("circle sort (recursive sc: ", InsertionNearSort::NAME, ", pre-order)"),
    &["circle sorts", "recursive, short circuit", InsertionNearSort::NAME, "pre-order"],
    drive_recursive::<PreOrder, ShortCircuit<InsertionNearSort>>
);
register_circle!(
    sc_left_mid_right,
    const_format::concatcp!("circle sort (recursive sc: ", InsertionNearSort::NAME, ", left-mid-right)"),
    &["circle sorts", "recursive, short circuit", InsertionNearSort::NAME, "left-mid-right"],
    drive_recursive::<LeftMidRight, ShortCircuit<InsertionNearSort>>
);
register_circle!(
    sc_right_mid_left,
    const_format::concatcp!("circle sort (recursive sc: ", InsertionNearSort::NAME, ", right-mid-left)"),
    &["circle sorts", "recursive, short circuit", InsertionNearSort::NAME, "right-mid-left"],
    drive_recursive::<RightMidLeft, ShortCircuit<InsertionNearSort>>
);
register_circle!(
    sc_post_order,
    const_format::concatcp!("circle sort (recursive sc: ", InsertionNearSort::NAME, ", post-order)"),
    &["circle sorts", "recursive, short circuit", InsertionNearSort::NAME, "post-order"],
    drive_recursive::<PostOrder, ShortCircuit<InsertionNearSort>>
);

// ---------------------------------------------------------------------------
// Bottom-up — convergence (run to completion)
// ---------------------------------------------------------------------------

register_circle!(
    decreasing, "circle sort (bottom-up decreasing)",
    &["circle sorts", "bottom-up", "decreasing"],
    drive_bottom_up::<Decreasing, Convergence>
);
register_circle!(
    increasing, "circle sort (bottom-up increasing)",
    &["circle sorts", "bottom-up", "increasing"],
    drive_bottom_up::<Increasing, Convergence>
);
register_circle!(
    shaker_dec_inc, "circle sort (bottom-up shaker dec\u{2192}inc)",
    &["circle sorts", "bottom-up", "shaker dec\u{2192}inc"],
    drive_bottom_up::<ShakerDecInc, Convergence>
);
register_circle!(
    shaker_inc_dec, "circle sort (bottom-up shaker inc\u{2192}dec)",
    &["circle sorts", "bottom-up", "shaker inc\u{2192}dec"],
    drive_bottom_up::<ShakerIncDec, Convergence>
);

// ---------------------------------------------------------------------------
// Bottom-up — short circuit (stop at log2(n), finish with S)
// ---------------------------------------------------------------------------

register_circle!(
    sc_decreasing,
    const_format::concatcp!("circle sort (bottom-up sc: ", InsertionNearSort::NAME, ", decreasing)"),
    &["circle sorts", "bottom-up, short circuit", InsertionNearSort::NAME, "decreasing"],
    drive_bottom_up::<Decreasing, ShortCircuit<InsertionNearSort>>
);
register_circle!(
    sc_increasing,
    const_format::concatcp!("circle sort (bottom-up sc: ", InsertionNearSort::NAME, ", increasing)"),
    &["circle sorts", "bottom-up, short circuit", InsertionNearSort::NAME, "increasing"],
    drive_bottom_up::<Increasing, ShortCircuit<InsertionNearSort>>
);
register_circle!(
    sc_shaker_dec_inc,
    const_format::concatcp!("circle sort (bottom-up sc: ", InsertionNearSort::NAME, ", shaker dec\u{2192}inc)"),
    &["circle sorts", "bottom-up, short circuit", InsertionNearSort::NAME, "shaker dec\u{2192}inc"],
    drive_bottom_up::<ShakerDecInc, ShortCircuit<InsertionNearSort>>
);
register_circle!(
    sc_shaker_inc_dec,
    const_format::concatcp!("circle sort (bottom-up sc: ", InsertionNearSort::NAME, ", shaker inc\u{2192}dec)"),
    &["circle sorts", "bottom-up, short circuit", InsertionNearSort::NAME, "shaker inc\u{2192}dec"],
    drive_bottom_up::<ShakerIncDec, ShortCircuit<InsertionNearSort>>
);
