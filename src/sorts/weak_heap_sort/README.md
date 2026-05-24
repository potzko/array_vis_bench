# weak_heap_sort

Weak heap sort — uses Dutton's *weak heap*, an implicit tree where each node satisfies `child > grandparent` (instead of the strong heap's `child < parent`). The structure trades the strong-heap invariant for ~50% fewer comparisons during the build phase.

## How a weak heap works

In a binary weak heap, every node carries a single *reverse bit*: if set, the left and right children are swapped. The invariant is that the right child is greater than its grandparent, not its parent. Building costs N − 1 comparisons (vs. roughly 2N for a binary heap); extraction is comparable. The reverse bits are stored in a parallel `Vec<u8>` that the visualiser observes as an aux array.

## Files

- `reverse_storage.rs` — bit-packed reverse-bit storage, registered as a logger-tracked `u8` aux array.
- `weak_heap_sort.rs` — `WeakHeapSort` family entry, implementing `HeapAlgorithm` with `State = Vec<u8>` for the reverse bits.

## Registration

`weak_heap_sort.rs` carries the `combo_codegen::family!` invocation; the cross-product is small because the algorithm has fewer independent axes than the strong-heap family.

## Visualisation

Every reverse-bit flip goes through `logger.write_data_u8` so the visualiser shows the bit array updating alongside the main array — the swap pattern is visibly different from binary heap sort despite the same number of element moves.
