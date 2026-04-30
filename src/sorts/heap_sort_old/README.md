# heap_sort

Heap sort variants — sorts based on the heap data structure, plus heap-quicksort hybrids.

## How heap sort works

Heap sort builds a max-heap from the array (every parent >= its children), then repeatedly extracts the maximum and places it at the end. Building the heap is O(N); each of the N extractions is O(log N), giving O(N log N) total. In-place and not stable.

## Subfolders

### `classic_heap_sorts/`

Standard binary-heap sorts with different branching factors:

- **Binary heap** (base-2) — the textbook version. Each node has 2 children.
- **Base-3 heap** — ternary heap. Each node has 3 children. Shallower tree but more comparisons per sift-down.
- **Base-16 heap** — 16-ary heap. Very shallow but many comparisons per level.
- **Base-256 heap** — 256-ary heap. Extreme branching factor experiment.

See [classic_heap_sorts/README.md](classic_heap_sorts/README.md).

### `quick_heap_sorts/`

Hybrids that use heap operations within a quicksort framework:

- **Heap-quick sort** — partitions like quicksort but uses a heap for one side.
- **Optimised variants** — reduced overhead versions.

See [quick_heap_sorts/README.md](quick_heap_sorts/README.md).

## Other files

- `heap_quick_sort.rs` — a standalone heap-quicksort hybrid at this level.
- `weak_heap_sort.rs` — **Weak heap sort**: uses a weak heap (relaxed heap where the heap property only holds for the right child). Requires fewer comparisons than standard heap sort — close to the theoretical N log N - N minimum.

## Status

Compiled but commented out of the active dispatch in `sorts/mod.rs`, pending migration to `sort_family!`.
