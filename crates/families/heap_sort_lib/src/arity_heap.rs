//! N-ary heap parameterized by arity and direction (compare + layout).
//!
//! One concrete heap type covers all 4 (compare × layout) combinations
//! by way of the [`Direction`] type parameter, and any arity via [`Arity`].

use std::marker::PhantomData;

use super::arity::Arity;
use super::compare::Compare;
use super::direction::Direction;
use super::heap::{Heap, HeapLayout};
use super::layout::Layout;
use sort_logger::SortLogger;

pub struct ArityHeap<A: Arity, D: Direction> {
    _phantom: PhantomData<(A, D)>,
}

impl<A: Arity, D: Direction> HeapLayout for ArityHeap<A, D> {
    type Compare = D::Compare;

    #[inline(always)]
    fn phys(i: usize, n: usize) -> usize {
        <D::Layout as Layout>::phys(i, n)
    }
}

impl<A: Arity, D: Direction> Heap for ArityHeap<A, D> {
    #[inline]
    fn heapify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        heap_size: usize,
        mut i: usize,
        logger: &mut U,
    ) {
        let n = arr.len();
        loop {
            let mut chosen = i;
            for k in 1..=A::N {
                let child = A::N * i + k;
                if child >= heap_size {
                    break;
                }
                let chosen_phys = <D::Layout as Layout>::phys(chosen, n);
                let child_phys = <D::Layout as Layout>::phys(child, n);
                if <D::Compare as Compare>::comes_first(logger, arr, child_phys, chosen_phys) {
                    chosen = child;
                }
            }
            if chosen == i {
                return;
            }
            let i_phys = <D::Layout as Layout>::phys(i, n);
            let chosen_phys = <D::Layout as Layout>::phys(chosen, n);
            logger.swap(arr, i_phys, chosen_phys);
            i = chosen;
        }
    }

    fn deep_heapify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        heap_size: usize,
        i: usize,
        logger: &mut U,
    ) {
        for k in 1..=A::N {
            let child = A::N * i + k;
            if child >= heap_size {
                break;
            }
            Self::deep_heapify(arr, heap_size, child, logger);
        }
        Self::heapify(arr, heap_size, i, logger);
    }

    #[inline]
    fn last_internal_node(n: usize) -> usize {
        // Parent of the last leaf. For an A-ary heap children of `i` live
        // at `A*i+1..=A*i+A`, so parent of node `k` is `(k - 1) / A` and
        // parent of the final leaf `n - 1` is `(n - 2) / A`.
        (n - 2) / A::N
    }

    fn layer_boundaries(n: usize) -> Vec<usize> {
        // B_0 = 1; B_k = B_{k-1} + A^k. Stop once the next boundary would
        // overshoot `n`.
        let mut boundaries = Vec::new();
        let mut b = 1usize;
        let mut layer_size = 1usize;
        while b < n {
            boundaries.push(b);
            layer_size = layer_size.saturating_mul(A::N);
            b = b.saturating_add(layer_size);
        }
        boundaries
    }
}

// ── Composable annotations ──────────────────────────────────────────
//
// Per-operation heapify cost: O(log_A N) ≈ O(log N) for any fixed
// arity A. The outer HeapSort composition multiplies by N
// extractions to get the overall sort complexity.

impl<A: super::arity::Arity, D: super::direction::Direction> array_vis_bench_traits::composable::HasTimeBounds
    for ArityHeap<A, D>
{
    const WORST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::LOG_N;
    const BEST: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::LOG_N;
    const AVERAGE: array_vis_bench_traits::Complexity = array_vis_bench_traits::Complexity::LOG_N;
}
