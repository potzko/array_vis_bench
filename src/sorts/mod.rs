pub mod bubble_sorts;
pub mod circle_sorts;
pub mod comb_sorts;
pub mod cycle_sorts;
pub mod fun_sorts;
pub mod heap_sort;
pub mod insertion_sorts;
pub mod merge_sorts;
pub mod quick_sorts;
pub mod shell_sorts;

use strum_macros::EnumIter;
#[derive(Debug, EnumIter)]
pub enum AnySort {
    Bubble(bubble_sorts::AnySort),
    Circle(circle_sorts::AnySort),
}
