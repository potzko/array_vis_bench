# Trait System

The project uses traits extensively to parameterise sort algorithms over independent strategy axes. This document covers each trait, its role, and how traits compose.

## `SortAlgo<T, U>` — the sort interface

```rust
pub trait SortAlgo<T: Ord + Copy, U: SortLogger<T>> {
    fn big_o() -> &'static str;
    fn name() -> &'static str;
    fn sort(arr: &mut [T], logger: &mut U);
    fn stable() -> bool;
}
```

Every sort implements this. The two type parameters allow the same code to run in benchmark mode (`T = usize, U = NoOpLogger`) and visualisation mode (`T = usize, U = VisualizerLogger` or `dyn SortLogger<usize>`).

**Design choice:** `sort` takes `&mut U` not `&mut dyn SortLogger<T>`. This means benchmark builds monomorphise the logger — every `log()` call is inlined and eliminated by the optimiser when `U = NoOpLogger`.

## `SortLogger<T>` — operation instrumentation

```rust
pub trait SortLogger<T: Copy + Ord> {
    fn log(&mut self, _: SortLog<T>) {}
    // 30+ default-implemented methods built on top of log():
    // cond_swap_lt, write_data, cmp_le_accross, create_aux_arr_t, ...
}
```

The single required method is `log`. All comparison, swap, write, and auxiliary-memory methods are built on top of it. Sorts call the high-level methods (`cond_swap_lt`, `write_data`, etc.) rather than `log` directly.

**dyn-compatibility:** Methods that are generic over a second type `U` (e.g. `swap<U>`, `cmp_le<U>`) are gated with `where Self: Sized` and excluded from the vtable. All T-specific methods work on `&mut dyn SortLogger<T>`.

### Implementations

| Type | Behaviour |
|---|---|
| `NoOpLogger` | `log()` is a no-op. Every call compiles away entirely. |
| `VisualizerLogger<T>` | Appends every `SortLog<T>` to a `Vec` for later GIF rendering. |
| `()` | Blanket no-op, convenient in unit tests. |

## `Rotation` — array rotation strategy

```rust
pub trait Rotation {
    const NAME: &'static str;
    fn rotate<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T], split_ind: usize, logger: &mut U,
    );
}
```

A rotation moves `arr[split..]` to the front and `arr[..split]` to the back. 11 implementations trade off between auxiliary memory, number of data movements, and cache locality.

**Used by:** `NaiveRotationMerge<R>` and `SmallerSideRotationMerge<R>` in the rotation merge sort family. The rotation algorithm is a type parameter, so different rotations produce different concrete sorts.

## `RotationMerge` — in-place merge strategy

```rust
pub trait RotationMerge {
    fn merge<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        arr: &mut [T], mid: usize, logger: &mut U,
    );
}
```

Merges `arr[..mid]` and `arr[mid..]` (both sorted) in-place using rotation.

### Implementations

| Type | Method |
|---|---|
| `NaiveRotationMerge<R>` | Linear scan from the left: binary-search for the insertion point, rotate, advance. O(N^2) comparisons worst case. |
| `SmallerSideRotationMerge<R>` | symMerge: picks a pivot from the shorter half, binary-searches its position in the other half, rotates, recurses on two sub-problems. O(N log N) moves, O(N log^2 N) comparisons. |

## `SmallSort` — base-case strategy for merge sorts

```rust
pub trait SmallSort {
    const THRESHOLD: usize;
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U);
}
```

Controls when a merge sort stops recursing and switches to a simpler sort.

| Type | Behaviour |
|---|---|
| `NoSmallSort` | `THRESHOLD = 0`. Recurse all the way to size 1. |
| `InsertionSmallSort<N>` | Switch to insertion sort for subarrays of size <= N. Typical value: 32. |

## `GapSequence` — shell sort gap strategy

```rust
pub trait GapSequence {
    const NAME: &'static str;
    const BIG_O: &'static str;
    fn gaps(len: usize) -> Vec<usize>;
}
```

Returns gap values in descending order for shell sort. 9 implementations from `Classic` (Shell 1959, O(N^2)) to `Pratt` (2^p * 3^q, O(N log^2 N)).

## `BranchingStrategy` — shell-shell/rod sort decomposition

```rust
pub trait BranchingStrategy {
    const NAME: &'static str;
    const BIG_O: &'static str;
    fn should_cut(virtual_len: usize) -> bool;
    fn branch(virtual_len: usize) -> usize;
    fn intermediate(virtual_len: usize) -> usize { 0 }
}
```

Controls how many interleaved sub-arrays each recursion level of shell-shell sort / rod sort splits into. 6 implementations from fixed (Classic: always 2) to adaptive (RootParity: sqrt(len)).

## `SortRegistry` — auto-registration trait

```rust
pub trait SortRegistry {
    fn register();
}
```

Implemented via `#[derive(SortRegistry)]` proc macro. The `register()` method inserts function pointers and metadata into the global registries. Called from a `#[ctor]` hook at program startup.

## How traits compose

A concrete rotation merge sort is parameterised over three independent trait axes plus two const-generic flags:

```rust
TopDownRotationMergeSort<
    InsertionSmallSort<32>,          // SmallSort
    SmallerSideRotationMerge<        // RotationMerge
        TrinityRotation              //   Rotation
    >,
    true                             // EARLY_EXIT
>
```

The `sort_family!` macro generates all meaningful combinations of these parameters automatically. Each combination becomes a separate registered sort with its own name, benchmark entry, and position in the navigation tree.
