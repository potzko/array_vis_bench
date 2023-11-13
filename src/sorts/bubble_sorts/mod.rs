pub mod bubble_sort;
pub mod odd_even_bubble_sort;
pub mod shaker_sort;

use strum_macros::EnumIter;
#[derive(Debug, EnumIter)]
pub enum AnySort {
    Bubble(bubble_sort::BubbleSort),
    OddEvenBubble(odd_even_bubble_sort::BubbleSort),
    Shaker(shaker_sort::BubbleSort),
}
impl Default for AnySort {
    fn default() -> Self {
        AnySort::Bubble(bubble_sort::BubbleSort {})
    }
}
