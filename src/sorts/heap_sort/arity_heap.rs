//! N-ary heap parameterized by arity and direction (compare + layout).
//!
//! One concrete heap type covers all 4 (compare × layout) combinations
//! by way of the [`Direction`] type parameter, and any arity via [`Arity`].

use std::marker::PhantomData;

use super::arity::Arity;
use super::compare::Compare;
use super::direction::Direction;
use super::heap::Heap;
use super::layout::Layout;
use crate::traits::log_traits::SortLogger;

pub struct ArityHeap<A: Arity, D: Direction> {
    _phantom: PhantomData<(A, D)>,
}

impl<A: Arity, D: Direction> Heap for ArityHeap<A, D> {
    type Arity = A;

    fn heapify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        heap_size: usize,
        i: usize,
        logger: &mut U,
    ) {
        let n = arr.len();
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
        if chosen != i {
            let i_phys = <D::Layout as Layout>::phys(i, n);
            let chosen_phys = <D::Layout as Layout>::phys(chosen, n);
            logger.swap(arr, i_phys, chosen_phys);
            Self::heapify(arr, heap_size, chosen, logger);
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

    fn swap<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        i: usize,
        j: usize,
        logger: &mut U,
    ) {
        let n = arr.len();
        let i_phys = <D::Layout as Layout>::phys(i, n);
        let j_phys = <D::Layout as Layout>::phys(j, n);
        logger.swap(arr, i_phys, j_phys);
    }

    #[inline(always)]
    fn phys(i: usize, n: usize) -> usize {
        <D::Layout as Layout>::phys(i, n)
    }
}
