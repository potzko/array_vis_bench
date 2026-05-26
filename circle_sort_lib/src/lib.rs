//! Circle sort family — recursive + bottom-up, with finishing strategies
//! and per-(variant) registrations into
//! `array_vis_bench_core::ALGORITHMS` via [`sequences`].

pub mod circle_sort_bottom_up;
pub mod circle_sort_recursive;
pub mod circle_sort_shaker_recursive;
pub mod directions;
pub mod finishing;
pub mod orderings;
pub mod sequences;

pub use circle_sort_bottom_up::CircleSortBottomUp;
pub use circle_sort_recursive::CircleSortRecursive;
pub use circle_sort_shaker_recursive::CircleSortShakerRecursive;
pub use sequences::CIRCLE_ENTRIES;

/// Iterates CIRCLE_ENTRIES at startup and registers each variant's path
/// with `sort_registry_core` so the menu tree contains every circle-sort
/// leaf. Algorithm dispatch goes through `ALGORITHMS`; this is the
/// path-registration side.
#[ctor::ctor]
fn register_circle_sorts() {
    for entry in CIRCLE_ENTRIES {
        let full: Vec<&str> = std::iter::once("sorts").chain(entry.path.iter().copied()).collect();
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, &full);
    }
}
