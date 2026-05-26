# heap_sort

Heap sort — build an n-ary heap in place, then repeatedly extract the root. Parameterised over arity, direction (heap built upward or downward in the array), comparison strategy, and a "deep heapify" axis that controls how aggressively each push-down compares with descendants.

## How heap sort works

A heap is an implicit tree stored contiguously in the array: child of node `i` is `arity * i + k` for the k-th of `arity` children. Heap sort:

1. **Build phase** — sift each non-leaf into place from the bottom up so the whole array satisfies the heap invariant.
2. **Extract phase** — swap the root with the last unsorted slot, shrink the heap by one, and push the new root down to restore the invariant. After N extractions the array is sorted.

## Variant axes

| Axis | Trait / type | Provided by |
|---|---|---|
| Arity (`A: Arity`) | branching factor `K` (binary, ternary, quaternary, …) | `arity.rs` |
| Direction (forward / reverse layout) | `Direction` + `Layout` | `direction.rs`, `layout.rs` |
| Comparison strategy | `Compare` (e.g. min-of-children to pick the swap target) | `compare.rs` |
| Deep heapify | how far each sift compares with descendants | `deep_heapify.rs`, `quick_deep_heapify.rs` |

## Files

- `heap.rs` / `arity_heap.rs` — the implicit-tree machinery, generic over arity and direction.
- `heap_algorithm.rs` — `HeapAlgorithm` trait shared with `weak_heap_sort` and consumed by `quick_heap_sort`. Default `sort` runs build-then-extract.
- `heap_partition.rs` — partitions used by deep-heapify variants.
- `heap_sort.rs` — `HeapSort` family entry point (`combo_codegen::family!`).
- `heap_sort_quick_build.rs` — variant that builds the heap with a quicksort-style pre-pass before extracting.

## Registration

`heap_sort.rs` and `heap_sort_quick_build.rs` each carry a `combo_codegen::family!` invocation enumerating the (arity × direction × compare × …) cross-product.
