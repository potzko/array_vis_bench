# merge_sorts

Merge sort variants — the most actively developed sort family in the project. This is the only family currently wired into the `sort_family!` registration system.

## How merge sort works

Merge sort divides the array into halves, recursively sorts each half, then *merges* the two sorted halves into one. The merge step is where all the variation lives: it can use auxiliary memory, rotate elements in-place, detect natural runs, or apply galloping heuristics.

## Variant axes

Every merge sort in this folder is parameterised along several independent axes:

| Axis | Options | Controlled by |
|---|---|---|
| **Decomposition** | Top-down (recursive), bottom-up (iterative), natural (run-adaptive), top-down-mirror (iterative matching recursive splits) | Separate files |
| **Merge strategy** | Auxiliary buffer, rotation (in-place) | `top_down.rs` vs `rotation.rs` |
| **Rotation algorithm** | 11 algorithms from `utils/rotation/` | `R: Rotation` type parameter |
| **Rotation merge strategy** | Naive (linear scan) vs symMerge (divide-and-conquer) | `rotation_merge.rs` |
| **Small-sort cutoff** | None, or insertion sort at threshold N | `small_sort.rs` |
| **Ping-pong buffering** | Swap src/dst each pass vs copy-back | `const PING_PONG: bool` |
| **Early exit** | Skip merge when halves already in order | `const EARLY_EXIT: bool` |
| **Galloping** (timsort only) | Exponential search when one run dominates | `const GALLOP: bool` |

The `combinations.rs` file uses `sort_family!` to generate all meaningful combinations and register them automatically.

## Files

### Core merge sort variants

- `naive.rs` — **NaiveMergeSort**: allocates fresh left/right sub-arrays at every recursion level. O(N log N) auxiliary space total. The textbook version.
- `top_down.rs` — **TopDownMergeSort**: single auxiliary buffer, recursive split. Supports ping-pong buffering (avoid copy-back) and early-exit optimisation.
- `bottom_up.rs` — **BottomUpMergeSort**: single auxiliary buffer, iterative doubling. Same optimisation flags as top-down.
- `top_down_mirror.rs` — **TopDownMirrorMergeSort**: iterative but produces the exact same merge sequence as top-down recursive. Uses Bresenham-style fixed-point stepping to match the `mid = (lo + hi) / 2` splits without recursion.
- `natural.rs` — **NaturalMergeSort**: detects maximal ascending/descending runs, reverses descending runs, then merges run pairs. Best-case O(N) on already-sorted input.
- `timsort.rs` — **TimSort**: adaptive hybrid — detects natural runs, extends short runs with binary insertion sort, maintains a merge stack with invariants, and optionally uses galloping mode for skipping during merge.

### In-place rotation merge sorts

- `rotation.rs` — **TopDownRotationMergeSort** and **BottomUpRotationMergeSort**: merge using rotation instead of an auxiliary buffer. Generic over `S: SmallSort`, `M: RotationMerge`, and `EARLY_EXIT`.
- `rotation_merge.rs` — Two merge strategies:
  - `NaiveRotationMerge<R>`: linear scan from the left, binary-search + rotate.
  - `SmallerSideRotationMerge<R>` (symMerge): divide-and-conquer merge with O(N log N) data movements and O(N log^2 N) comparisons. Pivots from the shorter half to avoid degenerate O(N^2) behaviour.

### Support modules

- `small_sort.rs` — `SmallSort` trait with two implementations: `NoSmallSort` (recurse to size 1) and `InsertionSmallSort<N>` (switch to insertion sort at threshold N).
- `utils.rs` — shared helpers: `merge_inplace` (two-way merge into destination), `insertion_sort`, `copy_across`, `lower_bound`, `upper_bound`, `reverse`.
- `combinations.rs` — `sort_family!` invocation that generates and registers all concrete merge sort variants.
