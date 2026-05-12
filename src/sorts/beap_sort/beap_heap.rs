//! Beap (bi-parental heap) — triangular-layout heap.
//!
//! Munro & Suwanda (1980). Array-backed like a binary heap but laid out
//! in layers of *increasing* size: layer `k` holds `k + 1` nodes starting
//! at index `T_k = k(k+1)/2`. Each non-edge node has *two* parents in the
//! layer above and *two* children in the layer below — hence
//! "bi-parental." The heap predicate is `parent ≥ child` (max-beap) on
//! every (parent, child) edge, same shape as binary but on the denser
//! parent set.
//!
//! ## Indexing
//!
//! For a node at logical index `i`, let `k = layer_of(i)`. Then
//!
//! ```text
//! children of i:  i + (k + 1)   (left child, layer k+1, position p)
//!                 i + (k + 2)   (right child, layer k+1, position p+1)
//! ```
//!
//! where `p` is the position within layer `k`. The children formula falls
//! out cleanly because `T_{k+1} = T_k + (k + 1)`.
//!
//! ## Build
//!
//! Iterative bottom-up sift-down works on the beap: even though each
//! node has two parents (so the structure is a DAG, not a tree),
//! processing indices in *descending* order and heapify-ing each is
//! enough to establish the predicate at every node.  Recursive subtree
//! builds *don't* fit beap — the DAG would re-visit each node `C(k, p)`
//! times — so [`Heap::deep_heapify`] is implemented in the same iterative
//! shape rather than the recurse-then-heapify pattern that fits binary
//! and n-ary heaps.

use std::marker::PhantomData;

use crate::sorts::heap_sort::compare::Compare;
use crate::sorts::heap_sort::direction::Direction;
use crate::sorts::heap_sort::heap::{Heap, HeapLayout};
use crate::sorts::heap_sort::layout::Layout;
use crate::traits::log_traits::SortLogger;

pub struct BeapHeap<D: Direction> {
    _phantom: PhantomData<D>,
}

/// Layer containing logical index `i`. `k(k+1)/2 ≤ i < (k+1)(k+2)/2` ⇔
/// `k = floor((sqrt(1 + 8i) - 1) / 2)`. Uses [`usize::isqrt`] for an
/// exact integer answer.
#[inline]
fn layer_of(i: usize) -> usize {
    // (1 + 8 * i).isqrt() = floor(sqrt(1 + 8i)); for i = 0 this is 1, for
    // i = 1 it's 3, etc. The subtraction can't underflow because the
    // expression is at least 1.
    let s = (1 + 8 * i).isqrt();
    (s - 1) / 2
}

impl<D: Direction> HeapLayout for BeapHeap<D> {
    type Compare = D::Compare;

    #[inline(always)]
    fn phys(i: usize, n: usize) -> usize {
        <D::Layout as Layout>::phys(i, n)
    }
}

impl<D: Direction> Heap for BeapHeap<D> {
    fn heapify<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T],
        heap_size: usize,
        i: usize,
        logger: &mut U,
    ) {
        let n = arr.len();
        let k = layer_of(i);
        let c1 = i + k + 1;
        let c2 = i + k + 2;

        let mut chosen = i;
        if c1 < heap_size {
            let chosen_phys = <D::Layout as Layout>::phys(chosen, n);
            let c1_phys = <D::Layout as Layout>::phys(c1, n);
            if <D::Compare as Compare>::comes_first(logger, arr, c1_phys, chosen_phys) {
                chosen = c1;
            }
        }
        if c2 < heap_size {
            let chosen_phys = <D::Layout as Layout>::phys(chosen, n);
            let c2_phys = <D::Layout as Layout>::phys(c2, n);
            if <D::Compare as Compare>::comes_first(logger, arr, c2_phys, chosen_phys) {
                chosen = c2;
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
        // The standard "recurse children, then heapify self" pattern
        // double-visits each node in a beap (two parents per non-edge
        // node). Iterating in descending logical order from heap_size − 1
        // down to `i` and heapify-ing each gives the same result without
        // exponential blowup.
        if i >= heap_size {
            return;
        }
        let last_internal = if heap_size >= 2 {
            Self::last_internal_node(heap_size)
        } else {
            return;
        };
        let start = last_internal.min(heap_size - 1);
        for j in (i..=start).rev() {
            Self::heapify(arr, heap_size, j, logger);
        }
    }

    #[inline]
    fn last_internal_node(n: usize) -> usize {
        // Tight bound: a parent of the last leaf. The last leaf is at
        // logical index `n - 1` in layer `k` at position `p`; its parents
        // live at `last - k - 1` (left, position p − 1, exists if p > 0)
        // and `last - k` (right, position p, exists if p < k). We want the
        // larger of the two — that's the rightmost internal node.
        let last = n - 1;
        let k = layer_of(last);
        let p = last - k * (k + 1) / 2;
        if p < k {
            // Right parent exists; it's the larger index.
            last - k
        } else {
            // Right edge of the layer (p == k): only the left parent
            // exists.
            last - k - 1
        }
    }

    fn layer_boundaries(n: usize) -> Vec<usize> {
        // Boundaries are T_1, T_2, T_3, … = 1, 3, 6, 10, 15, ….
        // Exclude T_0 = 0 and any T_k ≥ n.
        let mut boundaries = Vec::new();
        let mut k = 1usize;
        loop {
            let b = k * (k + 1) / 2;
            if b >= n {
                break;
            }
            boundaries.push(b);
            k += 1;
        }
        boundaries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_math_first_eleven() {
        // indices 0..11 → layers 0,1,1,2,2,2,3,3,3,3,4
        let expected = [0, 1, 1, 2, 2, 2, 3, 3, 3, 3, 4];
        for (i, &want) in expected.iter().enumerate() {
            assert_eq!(layer_of(i), want, "layer_of({i}) wrong");
        }
    }

    #[test]
    fn last_internal_examples() {
        type H = BeapHeap<crate::sorts::heap_sort::direction::MaxForward>;
        // n = 2: last=1 in layer 1, p=0 < k=1 → right parent at last - k = 0.
        assert_eq!(<H as Heap>::last_internal_node(2), 0);
        // n = 4: last=3 in layer 2, p=0 < k=2 → right parent at last - k = 1.
        // (Node 2 in layer 1 has children 4, 5 — both ≥ 4, so it's a leaf.)
        assert_eq!(<H as Heap>::last_internal_node(4), 1);
        // n = 7: last=6 in layer 3, p=0 < k=3 → right parent at last - k = 3.
        assert_eq!(<H as Heap>::last_internal_node(7), 3);
        // n = 10: last=9 in layer 3, p = 9 - T_3 = 3 = k → right edge,
        // left parent at last - k - 1 = 5.
        assert_eq!(<H as Heap>::last_internal_node(10), 5);
        // n = 11: last=10 in layer 4, p=0 < k=4 → right parent at last - k = 6.
        assert_eq!(<H as Heap>::last_internal_node(11), 6);
    }

    #[test]
    fn layer_boundaries_examples() {
        type H = BeapHeap<crate::sorts::heap_sort::direction::MaxForward>;
        assert_eq!(<H as Heap>::layer_boundaries(10), vec![1, 3, 6]);
        // n = 11: 10 is *in* layer 4, so T_4 = 10 is included as boundary.
        assert_eq!(<H as Heap>::layer_boundaries(11), vec![1, 3, 6, 10]);
        assert_eq!(<H as Heap>::layer_boundaries(1), Vec::<usize>::new());
    }
}
