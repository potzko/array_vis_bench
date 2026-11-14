//! `SmoothSort<D, S, F, B>`, the sort: a `HeapAlgorithm` whose heap is a forest
//! of trees, built by `B`, whose greatest element is always last, so that
//! taking it off is the whole of the swap to the end and what is left to do is
//! to set the two subtrees of the tree it was the root of among the roots.

use core::marker::PhantomData;

use array_vis_bench_traits::composable::{HasSpace, HasStability, HasTimeBounds};
use array_vis_bench_traits::Complexity;
use heap_sort_lib::direction::Direction;
use heap_sort_lib::heap_algorithm::HeapAlgorithm;
use heap_sort_lib::layout::Layout;
use sort_logger::SortLogger;

use crate::build::Build;
use crate::heap::SmoothHeap;
use crate::shape::{Forest, Shape};
use crate::sift::Sift;

/// `D` is the heap direction, `S` the shape of the trees, `F` the sift and `B`
/// the build.
pub struct SmoothSort<D: Direction, S: Shape, F: Sift, B: Build>(PhantomData<(D, S, F, B)>);

impl<D: Direction, S: Shape, F: Sift, B: Build> HeapAlgorithm for SmoothSort<D, S, F, B> {
    type State = Forest;

    fn new_state<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_n: usize, _log: &mut U) -> Forest { Forest::default() }

    // The greatest element, which is the last.
    fn root_phys(n: usize) -> usize { <D::Layout as Layout>::phys(n - 1, n) }

    fn build<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, log: &mut U) {
        B::build::<D, S, F, T, U>(arr, forest, log);
    }

    // The last element is the greatest already, where the sort wants it.
    fn swap_root_to_end<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _forest: &mut Forest, _heap_size: usize, _log: &mut U) {}

    // The heap shrinks to `heap_size`, so the greatest, at that index, is off.
    fn push_down<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], forest: &mut Forest, heap_size: usize, log: &mut U) {
        SmoothHeap::<D, S, F>::remove_last(arr, forest, heap_size + 1, log);
    }
}

impl<D: Direction, S: Shape, F: Sift, B: Build> SmoothSort<D, S, F, B> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], log: &mut U) {
        <Self as HeapAlgorithm>::sort(arr, log)
    }
}

array_vis_bench_traits::direct_sort!(SmoothSort<D, S, F, B>; D: Direction, S: Shape, F: Sift, B: Build);

// A sorted input is a linear number of comparisons, each element joining the
// forest without a trade and leaving without one, and the rest is the heap
// sort's: a sift is a tree's depth, a forest's roots are a logarithm, and every
// element is added and removed once. The forest is a logarithm of orders.
impl<D: Direction, S: Shape, F: Sift, B: Build> HasTimeBounds for SmoothSort<D, S, F, B> {
    const WORST: Complexity = Complexity::N_LOG_N;
    const BEST: Complexity = Complexity::N1;
    const AVERAGE: Complexity = Complexity::N_LOG_N;
}

impl<D: Direction, S: Shape, F: Sift, B: Build> HasSpace for SmoothSort<D, S, F, B> {
    const SPACE: Complexity = Complexity::LOG_N;
}

impl<D: Direction, S: Shape, F: Sift, B: Build> HasStability for SmoothSort<D, S, F, B> {
    const STABLE: bool = false;
}
