pub mod circle_sort_bottom_up;
pub mod circle_sort_bottom_up_increasing_size;
pub mod circle_sort_bottom_up_optimised;
pub mod circle_sort_recursive;
pub mod circle_sort_recursive_increasing_size;
pub mod circle_sort_recursive_optimised;
pub mod shaker_circle_sort;
pub mod shaker_circle_sort_bottom_up;
pub mod shaker_circle_sort_bottom_up_b;
pub mod stooge_circle_sort;
pub mod stooge_circle_sort_reversed;

use strum_macros::EnumIter;
#[derive(Debug, EnumIter)]
pub enum AnySort {
    Circle(circle_sort_recursive::CircleSort),
    CircleOptimised(circle_sort_recursive_optimised::CircleSort),
    CircleSortIncreesing(circle_sort_recursive_increasing_size::CircleSort),
    CircleBottomUp(circle_sort_bottom_up::CircleSort),
    CircleBottomUpOptimised(circle_sort_bottom_up_optimised::CircleSort),
    CircleBottomUpInreesing(circle_sort_bottom_up_increasing_size::CircleSort),
    CircleShaker(shaker_circle_sort::CircleSort),
    CircleShakerBottomUpA(shaker_circle_sort_bottom_up::CircleSort),
    CircleShakerBottomUpB(shaker_circle_sort_bottom_up_b::CircleSort),
    StoogeShakerSort(stooge_circle_sort::CircleSort),
    StoogeShakerSortReversed(stooge_circle_sort_reversed::CircleSort),
}
impl Default for AnySort {
    fn default() -> Self {
        AnySort::Circle(circle_sort_recursive::CircleSort {})
    }
}
