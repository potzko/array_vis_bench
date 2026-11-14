//! How the forest is first made: `Insert` grows it one element at a time, each
//! element rectified into the roots as it comes.

use heap_sort_lib::direction::Direction;
use sort_logger::SortLogger;

use crate::heap::SmoothHeap;
use crate::shape::{Forest, Shape};
use crate::sift::Sift;

pub trait Build {
    /// Makes a forest of the whole array, `forest` empty on the way in.
    fn build<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, log: &mut U);
}

/// Every element rectified as it is added.
pub struct Insert;

impl Build for Insert {
    fn build<D: Direction, S: Shape, F: Sift, T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, log: &mut U) {
        for at in 0..arr.len() {
            SmoothHeap::<D, S, F>::add(arr, forest, at, log);
            SmoothHeap::<D, S, F>::rectify(arr, forest.orders(), at, log);
        }
    }
}
