//! Re-export shim. The `QuickSelect` trait lives in
//! `array_vis_bench_traits`; `RecursiveQuickSelect` /
//! `IterativeQuickSelect` (and their composable annotations) live in
//! `quick_select_lib`.

pub use array_vis_bench_traits::QuickSelect;
pub use quick_select_lib::{IterativeQuickSelect, RecursiveQuickSelect};
