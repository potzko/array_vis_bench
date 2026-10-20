//! Circle sort family, recursive + bottom-up, with finishing strategies.
//! The catalog emits the variants from `circle.spec` and `circle.avbs`.

pub mod circle_sort_bottom_up;
pub mod circle_sort_recursive;
pub mod circle_sort_shaker_recursive;
pub mod directions;
pub mod drivers;
pub mod orderings;

pub use circle_sort_bottom_up::CircleSortBottomUp;
pub use circle_sort_recursive::CircleSortRecursive;
pub use circle_sort_shaker_recursive::CircleSortShakerRecursive;

// Spec-system driver heads + their composable finish axis.
pub use drivers::{
    CircleBottomUpOf, CircleFinish, CircleRecursiveOf, ConvergeFinish, InsertionShortCircuit,
};
// Slot-filler types referenced by the spec `uses` paths
// (`circle_sort_lib::PreOrder`, …).
pub use directions::{Decreasing, Increasing, ShakerDecInc, ShakerIncDec};
pub use orderings::{LeftMidRight, PostOrder, PreOrder, RightMidLeft};
