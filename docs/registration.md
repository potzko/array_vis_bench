# Sort Registration System

How sorts announce their existence to the framework without any central list.

## The problem

With 50+ sort variants (and growing), maintaining a central `match` statement or array of sorts would be brittle and error-prone. Every new sort would require editing a file far from the sort's own code.

## Solution: self-registration at startup

Every sort registers itself during program initialisation, using two complementary mechanisms.

## Mechanism 1: `create_sort!` + `#[derive(SortRegistry)]`

The older, per-sort approach. Used by sorts that were written before `sort_family!` existed.

```rust
// In bubble_sort.rs:
fn bubble_sort<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    // ... algorithm ...
}

create_sort!(bubble_sort, "bubble sort", "O(N^2)", true);
```

`create_sort!` generates:

1. **`SortImp<T, U>`** — a type implementing `SortAlgo<T, U>` that delegates to the sort function.
2. **`SortReg`** with `#[derive(SortRegistry)]` — triggers the proc macro, which generates:
   - A monomorphic `fn __sort_fn_*(arr: &mut [usize], logger: &mut NoOpLogger)` — fully inlinable.
   - A `#[ctor]` hook that inserts the function pointer into `SORT_REGISTRY` and calls `sort_registry_core::register_sort()`.
3. **A `linkme` distributed-slice entry** — `static __BENCH_SORT_ENTRY: SortBenchEntry` collected into `BENCH_SORTS` at link time.

## Mechanism 2: `sort_family!` macro

The newer, combinatoric approach. Designed for parameterised sorts with many variants.

```rust
sort_family! {
    type Sort = TopDownMergeSort<{SmallSort}, {PingPong}, {EarlyExit}>;

    SmallSort {
        NoSmallSort => "no-ss"
        InsertionSmallSort<32> => "ins-32"
    }

    PingPong { false => "cb"  true => "pp" }
    EarlyExit { false => "no-ee"  true => "ee" }

    name   = "merge sort top-down {SmallSort} {PingPong} {EarlyExit}";
    big_o  = "O(N log N)";
    stable = true;
    path   = ["merge sorts", "top-down", "{SmallSort}", "{PingPong}"];
}
```

This generates **2 × 2 × 2 = 8** concrete sort registrations, each with:
- A `linkme` `BENCH_SORTS` entry.
- A `#[ctor]` hook for `SORT_REGISTRY` and `SORT_VIS_REGISTRY`.
- Metadata in `sort_registry_core` with a navigation path for the tree menu.

For rotation merge sorts with 11 rotation algorithms × 2 merge strategies × 2 small-sort options × 2 early-exit options, this generates 80+ variants from a single macro invocation.

## Mechanism 3: Manual `#[ctor]` + distributed slice

Some families (shell sorts, comb sorts, rod sorts, circle sorts) use a middle-ground approach:

1. Define a distributed slice of registration entries in the family's `sequences.rs` or `branching.rs`.
2. Each entry contains the sort's name, function pointers, and metadata.
3. A single `#[ctor]` in `combinations.rs` iterates the slice and registers everything into `SORT_REGISTRY`.

```rust
// In shell_sorts/combinations.rs:
#[ctor::ctor]
fn register_shell_sorts() {
    for entry in GAP_SEQUENCES {
        registry.insert(entry.name.to_string(), entry.sort_fn);
        sort_registry_core::register_sort_path(entry.name, entry.big_o, false, entry.path);
    }
}
```

## Two registries

| Registry | Type | Purpose |
|---|---|---|
| `SORT_REGISTRY` | `HashMap<String, fn(&mut [usize], &mut NoOpLogger)>` | Benchmark dispatch. Monomorphic function pointers — no trait objects, fully inlinable. |
| `SORT_VIS_REGISTRY` | `HashMap<String, fn(&mut [usize], &mut dyn SortLogger<usize>)>` | Visualisation dispatch. Accepts `dyn SortLogger` for recording operations. |

Both are `lazy_static` `Mutex<HashMap>`s populated by `#[ctor]` hooks before `main` runs.

## `BENCH_SORTS` distributed slice

A `linkme` distributed slice of `SortBenchEntry` structs. Unlike the `HashMap` registries, this is assembled at link time — no runtime insertion needed. The Criterion benchmark iterates this slice directly.

## Navigation tree

`sort_registry_core` maintains a `SortTree` built from the navigation paths provided during registration. The interactive CLI walks this tree to present a hierarchical sort-selection menu.

```
merge sorts/
  top-down/
    no-ss/
      cb/
        "merge sort top-down no-ss cb no-ee"
        "merge sort top-down no-ss cb ee"
      pp/
        ...
```
