# Compile-Time Strategy Pattern Refactor

## Goal

Replace runtime string dispatch (match on `&str` at every recursive call) with
zero-sized strategy types so the compiler monomorphizes every combination into
its own fully-inlined, fully-optimized function. Registration still happens
automatically at startup — zero manual central lists.

---

## Core Ideas

### Strategy traits
Each "axis of variation" in a sort family becomes a trait with:
- `const NAME: &'static str` — used to build the sort's display name at compile time
- `const BIG_O: &'static str`
- The strategy's behaviour as an associated function

### Generic sort structs
`ShellSort<Seq>`, `QuickSort<Part, Piv>` etc. are generic over strategy types.
The sort algorithm is an inherent `pub fn sort<T, U>(arr, logger)` — no `SortAlgo`
impl on the generic struct itself (avoids the `name() -> &'static str` problem
with associated consts on type parameters).

### Registration macro
A declarative macro (`register_shell_sort!`, later `register_combinations!`)
wraps each concrete instantiation in its own `mod`:

```rust
register_shell_sort!(classic, Classic);
// expands to:
pub mod classic {
    const SORT_NAME: &'static str = const_format::concatcp!("shell sort<sequence: ", Classic::NAME, ">");
    #[derive(sort_registry_macro::SortRegistry)]
    pub struct SortReg;
    impl SortAlgo<usize, NoOpLogger> for SortReg { ... }
    // + linkme bench slice entry
}
```

`SORT_NAME` is a compile-time `&'static str` built with `const_format::concatcp!`
from a concrete type's `NAME` const — no runtime allocation, satisfies `linkme`.

`#[derive(SortRegistry)]` generates a `#[ctor]` startup function that inserts
name → fn-pointer into `SORT_REGISTRY` and adds the name to `SORT_NAMES`.

### Dispatch
`fn_sort` in each family module matches on `SORT_NAME` constants from the
combination modules (using `if name == combinations::classic::SORT_NAME` guards,
since `&str` constants can't be used as bare match arm patterns).

---

## Implementation Phases

### Phase 1 — Insertion Sort + Shell Sort (CURRENT)

Proves the full pipeline with two simple cases before tackling the 2D
cross-product of quick sort.

**Status:** Insertion sort unchanged (already uses `create_sort!`).
Shell sort rewritten with `GapSequence` trait and five sequences.
All other sorts **disconnected** (see below).

Registered sort names produced by Phase 1:
```
insertion sort
shell sort<sequence: classic>
shell sort<sequence: knuth>
shell sort<sequence: hibbard>
shell sort<sequence: sedgewick>
shell sort<sequence: ciura>
```

### Phase 2 — Quick Sort (TODO)

Two strategy axes → N×M combinations auto-registered.

**Strategy traits to define:**
- `PivotStrategy<T,U>` — `choose_pivot(arr, logger) -> usize`
  - Concrete types: `FirstElement`, `LastElement`, `MiddleElement`,
    `MedianOfThree`, `FirstThree`, `ThreeLast`, `MedianOfMedians`
- `PartitionStrategy<T,U>` — `partition(arr, logger, pivot_idx) -> usize`
  - Concrete types: `PartitionLeftLeft`, `PartitionLeftRightPointers`

**The optimized flag** — use `const OPT: bool` as a const generic on the
recursive engine `fn quick_sort_inner<T, U, Part, Piv, const OPT: bool>`.
The compiler eliminates the unused branch per monomorphization.

**Registration macro** — extend `register_shell_sort!` into a general
`register_combinations!` that accepts two lists and cross-products them,
also emitting a `pub fn dispatch_by_name<T,U>` function (a generated match
over all N×M combination names).

**Files to create:**
- `src/sorts/quick_sorts/strategies.rs`
- `src/sorts/quick_sorts/quick_sort_generic.rs`
- `src/sorts/quick_sorts/combinations.rs`

**Files to delete** after Phase 2 is verified:
- `src/sorts/quick_sorts/strategy_registry.rs`
- `src/sorts/quick_sorts/auto_register.rs`
- `src/sorts/quick_sorts/generic_quick_sort.rs`
- `src/sorts/quick_sorts/pivot_strategies.rs`
- `src/sorts/quick_sorts/partition_strategies.rs`

### Phase 3 — Reconnect Remaining Sorts (TODO)

Reimplement bubble, merge, heap, cycle, comb, circle, fun sorts using the new
preferred API (`logger.swap`, `logger.cond_swap_gt` etc. — no raw `logger.write`
+ `logger.cmp_gt_data` pattern). Register via `create_sort!` as before.

---

## Disconnected Sorts

The following sort families are **commented out** in `src/sorts/mod.rs`.
Their source files are untouched. To reconnect a family:

1. Uncomment `pub mod <family>;` in `src/sorts/mod.rs`
2. Add its arm back to `fn_sort` and `options` in `src/sorts/mod.rs`
3. Add name mappings back to `create_sort_choice` in `src/main.rs`

| Family | Module | Note |
|---|---|---|
| Bubble sorts | `bubble_sorts` | Old API — reimplement in Phase 3 |
| Circle sorts | `circle_sorts` | Old API — reimplement in Phase 3 |
| Comb sorts | `comb_sorts` | Old API — reimplement in Phase 3 |
| Cycle sorts | `cycle_sorts` | Old API — reimplement in Phase 3 |
| Fun sorts | `fun_sorts` | Old API — reimplement in Phase 3 |
| Heap sorts | `heap_sort` | Old API — reimplement in Phase 3 |
| Merge sorts | `merge_sorts` | Old API — reimplement in Phase 3 |
| Quick sorts | `quick_sorts` | Pending Phase 2 redesign |
| Old shell variants | `classic_shell_sorts`, `shell_shell_sorts` | Superseded by Phase 1 |

---

## Key Constraint: `name() -> &'static str`

`SortAlgo` requires `name()` to return `&'static str`. This means sort names
must be compile-time constants. The solution:

- Each strategy type has `const NAME: &'static str`
- The registration macro defines `const SORT_NAME` using `const_format::concatcp!`
  with **concrete** types (not type parameters) — this always evaluates correctly
- The generic sort structs (`ShellSort<Seq>`) do NOT implement `SortAlgo` directly;
  only the concrete wrappers generated by the registration macro do

This sidesteps the "associated const of a type parameter in concatcp!" problem
entirely, since the macro always substitutes concrete type paths.

---

## Preferred Logger API

Always use the high-level logger methods. Avoid raw `cmp_*` + `write` patterns.

**Preferred:**
```rust
logger.swap(arr, i, j);
logger.cond_swap_gt(arr, i, j);  // swaps and returns true if arr[i] > arr[j]
logger.cond_swap_lt(arr, i, j);  // swaps and returns true if arr[i] < arr[j]
```

**Avoid:**
```rust
if logger.cmp_gt_data(arr, j - 1, temp) {
    logger.write(arr, j, j - 1);
}
```
