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
// Registration macros
// ---------------------------------------------------------------------------

// Convergence variants (run to completion, no finishing sort)
macro_rules! register_circle_recursive {
    ($mod:ident, $sort_name:literal, $path:expr, $Order:ident) => {
        mod $mod {
            use super::{$Order, Convergence, CircleEntry, CIRCLE_ENTRIES};
            use crate::sorts::circle_sorts::finishing::drive_recursive;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const SORT_NAME: &str = $sort_name;
            const PATH: &[&str] = $path;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) {
                drive_recursive::<$Order, Convergence>(arr, logger);
            }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
                drive_recursive::<$Order, Convergence>(arr, logger);
            }
            fn bench(arr: &mut [usize]) {
                let mut l = NoOpLogger;
                drive_recursive::<$Order, Convergence>(arr, &mut l);
            }

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
        }
    };
}

macro_rules! register_circle_bottom_up {
    ($mod:ident, $sort_name:literal, $path:expr, $Dir:ident) => {
        mod $mod {
            use super::{$Dir, Convergence, CircleEntry, CIRCLE_ENTRIES};
            use crate::sorts::circle_sorts::finishing::drive_bottom_up;
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const SORT_NAME: &str = $sort_name;
            const PATH: &[&str] = $path;

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) {
                drive_bottom_up::<$Dir, Convergence>(arr, logger);
            }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
                drive_bottom_up::<$Dir, Convergence>(arr, logger);
            }
            fn bench(arr: &mut [usize]) {
                let mut l = NoOpLogger;
                drive_bottom_up::<$Dir, Convergence>(arr, &mut l);
            }

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
        }
    };
}

// Short-circuit variants: stop after log2(n) passes, finish with S
macro_rules! register_circle_recursive_sc {
    ($mod:ident, $order_display:literal, $Order:ident, $S:ident) => {
        mod $mod {
            use super::{$Order, $S, ShortCircuit, CircleEntry, CIRCLE_ENTRIES};
            use crate::sorts::circle_sorts::finishing::{drive_recursive, NearSortedSort};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const SORT_NAME: &str = const_format::concatcp!(
                "circle sort (recursive sc: ", $S::NAME, ", ", $order_display, ")"
            );
            const PATH: &[&str] = &[
                "circle sorts", "recursive, short circuit", $S::NAME, $order_display
            ];

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) {
                drive_recursive::<$Order, ShortCircuit<$S>>(arr, logger);
            }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
                drive_recursive::<$Order, ShortCircuit<$S>>(arr, logger);
            }
            fn bench(arr: &mut [usize]) {
                let mut l = NoOpLogger;
                drive_recursive::<$Order, ShortCircuit<$S>>(arr, &mut l);
            }

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
        }
    };
}

macro_rules! register_circle_bottom_up_sc {
    ($mod:ident, $dir_display:literal, $Dir:ident, $S:ident) => {
        mod $mod {
            use super::{$Dir, $S, ShortCircuit, CircleEntry, CIRCLE_ENTRIES};
            use crate::sorts::circle_sorts::finishing::{drive_bottom_up, NearSortedSort};
            use crate::traits::log_traits::{NoOpLogger, SortLogger};

            const SORT_NAME: &str = const_format::concatcp!(
                "circle sort (bottom-up sc: ", $S::NAME, ", ", $dir_display, ")"
            );
            const PATH: &[&str] = &[
                "circle sorts", "bottom-up, short circuit", $S::NAME, $dir_display
            ];

            fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) {
                drive_bottom_up::<$Dir, ShortCircuit<$S>>(arr, logger);
            }
            fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
                drive_bottom_up::<$Dir, ShortCircuit<$S>>(arr, logger);
            }
            fn bench(arr: &mut [usize]) {
                let mut l = NoOpLogger;
                drive_bottom_up::<$Dir, ShortCircuit<$S>>(arr, &mut l);
            }

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
        }
    };
}

// Shaker recursive: non-generic over Order, registered inline.
mod shaker_recursive {
    use super::{CircleEntry, CIRCLE_ENTRIES};
    use crate::sorts::circle_sorts::circle_sort_shaker_recursive::CircleSortShakerRecursive;
    use crate::traits::log_traits::{NoOpLogger, SortLogger};

    const SORT_NAME: &str = "circle sort (recursive shaker)";
    const PATH: &[&str] = &["circle sorts", "recursive", "shaker"];

    fn sort_fn(arr: &mut [usize], logger: &mut NoOpLogger) {
        CircleSortShakerRecursive::sort(arr, logger);
    }
    fn sort_vis(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) {
        CircleSortShakerRecursive::sort(arr, logger);
    }
    fn bench(arr: &mut [usize]) {
        let mut l = NoOpLogger;
        CircleSortShakerRecursive::sort(arr, &mut l);
    }

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
}

// ---------------------------------------------------------------------------
// Recursive — convergence (run to completion)
// ---------------------------------------------------------------------------

register_circle_recursive!(
    pre_order, "circle sort (recursive pre-order)",
    &["circle sorts", "recursive", "pre-order"],
    PreOrder
);
register_circle_recursive!(
    left_mid_right, "circle sort (recursive left-mid-right)",
    &["circle sorts", "recursive", "left-mid-right"],
    LeftMidRight
);
register_circle_recursive!(
    right_mid_left, "circle sort (recursive right-mid-left)",
    &["circle sorts", "recursive", "right-mid-left"],
    RightMidLeft
);
register_circle_recursive!(
    post_order, "circle sort (recursive post-order)",
    &["circle sorts", "recursive", "post-order"],
    PostOrder
);

// ---------------------------------------------------------------------------
// Recursive — short circuit (stop at log2(n), finish with S)
// ---------------------------------------------------------------------------

register_circle_recursive_sc!(sc_pre_order,      "pre-order",      PreOrder,      InsertionNearSort);
register_circle_recursive_sc!(sc_left_mid_right,  "left-mid-right", LeftMidRight,  InsertionNearSort);
register_circle_recursive_sc!(sc_right_mid_left,  "right-mid-left", RightMidLeft,  InsertionNearSort);
register_circle_recursive_sc!(sc_post_order,      "post-order",     PostOrder,     InsertionNearSort);

// ---------------------------------------------------------------------------
// Bottom-up — convergence (run to completion)
// ---------------------------------------------------------------------------

register_circle_bottom_up!(
    decreasing, "circle sort (bottom-up decreasing)",
    &["circle sorts", "bottom-up", "decreasing"],
    Decreasing
);
register_circle_bottom_up!(
    increasing, "circle sort (bottom-up increasing)",
    &["circle sorts", "bottom-up", "increasing"],
    Increasing
);
register_circle_bottom_up!(
    shaker_dec_inc, "circle sort (bottom-up shaker dec\u{2192}inc)",
    &["circle sorts", "bottom-up", "shaker dec\u{2192}inc"],
    ShakerDecInc
);
register_circle_bottom_up!(
    shaker_inc_dec, "circle sort (bottom-up shaker inc\u{2192}dec)",
    &["circle sorts", "bottom-up", "shaker inc\u{2192}dec"],
    ShakerIncDec
);

// ---------------------------------------------------------------------------
// Bottom-up — short circuit (stop at log2(n), finish with S)
// ---------------------------------------------------------------------------

register_circle_bottom_up_sc!(sc_decreasing,     "decreasing",            Decreasing,   InsertionNearSort);
register_circle_bottom_up_sc!(sc_increasing,     "increasing",            Increasing,   InsertionNearSort);
register_circle_bottom_up_sc!(sc_shaker_dec_inc, "shaker dec\u{2192}inc", ShakerDecInc, InsertionNearSort);
register_circle_bottom_up_sc!(sc_shaker_inc_dec, "shaker inc\u{2192}dec", ShakerIncDec, InsertionNearSort);
