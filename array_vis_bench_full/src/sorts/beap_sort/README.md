# beap_sort

Beap sort — sort by building a *bi-parental heap* (beap) and extracting the root repeatedly.

## How beap sort works

A beap arranges elements in a triangular structure where every node (except those on the borders) has two parents and two children. Insertions and extractions traverse from one corner along O(√N) diagonal paths instead of O(log N), giving the beap O(√N) per-operation behaviour and O(N^1.5) overall — slower than a binary heap but with a distinctive triangular access pattern.

## Files

- `beap_heap.rs` — the underlying beap data structure: coordinates, parents, children, sift up/down.
- `beap_sort.rs` — `BeapSort` family entry, built on `HeapAlgorithm`.
- `beap_sort_quick_build.rs` — variant that builds the beap with a quicksort-style pre-pass before the extract phase.

## Registration

Both `beap_sort.rs` and `beap_sort_quick_build.rs` use `combo_codegen::family!` and appear under `sorts / beap sort /` in the menu.
