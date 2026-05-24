# Adding a New Sort

How to add a new sort to the project. Most paths are one file edit.

## Option 1: Single-leaf sort

For a one-off sort with no parameterisation. Use `sort_registry_macro::sort_family!`.

```rust
// src/sorts/bubble_sorts/my_sort.rs
use crate::traits::log_traits::SortLogger;

pub struct MySort;

impl MySort {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        // Algorithm body.
        // Use logger methods for every comparison and write:
        //   logger.cond_swap_gt(arr, i, j)
        //   logger.write_data(arr, i, value)
        //   logger.cmp_lt(arr, i, j)
    }
}

sort_registry_macro::sort_family! {
    type Sort = MySort;
    name        = "my sort";
    big_o       = "O(N^2)";
    stable      = true;
    direct_sort = true;
    path        = ["bubble sorts", "my sort"];
}
```

Add the module in the family's `mod.rs`:

```rust
pub mod my_sort;
```

Done. The macro produces a `bench_registry::AlgorithmEntry`, a `#[ctor]` that registers the menu path, and a `linkme` entry — no further wiring needed.

## Option 2: Parameterised family

When a sort has independent strategy axes (small-sort cutoff, gap sequence, rotation algorithm, …) prefer `combo_codegen::family!`.

### 1. Define the strategy trait + annotate concrete impls

```rust
pub trait MyStrategy {
    const NAME: &'static str;
    fn do_thing<T, U: ?Sized + SortLogger<T>>(/* ... */);
}

pub struct StrategyA;
combo_codegen::component!(MyStrategy, StrategyA, "a");

pub struct StrategyB;
combo_codegen::component!(MyStrategy, StrategyB, "b");
```

The `component!` annotations are zero-cost markers; the build script scans for them.

### 2. Implement the sort generically

```rust
pub struct MySort<S: MyStrategy>(PhantomData<S>);

impl<S: MyStrategy> MySort<S> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        // Use S::do_thing(...) for the parameterised step
    }
}
```

### 3. Declare the family

```rust
combo_codegen::family!(
    type = MySort<{S}>,
    uses = [
        "crate::sorts::my_sorts::my_sort::MySort",
        "crate::sorts::my_sorts::strategies::{StrategyA, StrategyB}",
    ],
    S: MyStrategy,
    name        = "my sort",
    big_o       = "O(N log N)",
    stable      = false,
    direct_sort = true,
    path        = ["my sorts", "{S}"],
);
```

Include the generated combinations in `mod.rs`:

```rust
pub mod combinations {
    include!(concat!(env!("OUT_DIR"), "/my_sorts_combinations.rs"));
}
```

That's it. One `AlgorithmEntry` will be emitted per concrete `MySort<StrategyA>`, `MySort<StrategyB>`, … and they all show up in `bench_registry::ALGORITHMS`.

## Rules for sort implementations

### Always route through the logger

Every comparison and data movement must go through `SortLogger` methods. Direct array access (`arr[i] = arr[j]`, `arr.swap(i, j)`) silently works for benchmarks but produces no visualisation events. Reading a value to a temporary (`let tmp = arr[i]`) is fine — only writes and comparisons need logging.

Preferred:

```rust
logger.cond_swap_lt(arr, i, j);          // swap if arr[i] < arr[j]
logger.cond_swap_gt(arr, i, j);          // swap if arr[i] > arr[j]
logger.write_data(arr, i, value);        // arr[i] = value, logged
logger.write_accross(src, i, dst, j);    // dst[j] = src[i], logged
logger.cmp_le_accross(a, i, b, j);       // compare across two arrays
logger.create_aux_arr_t(len) / free_aux_arr_t(&buf)  // aux-memory tracking
```

### Generic over `<T: Ord + Copy, U: ?Sized + SortLogger<T>>`

Use `U: ?Sized` so the same code works with the concrete `NoOpLogger` (bench path) and `dyn SortLogger<T>` (visualiser path). Only drop `?Sized` if you call the trait's `where Self: Sized` cross-`T` helpers.

### Const-generic booleans for flags

```rust
pub struct MySort<S: SmallSort, const PING_PONG: bool, const EARLY_EXIT: bool> { ... }
```

Each `(flag, …)` combination monomorphises to its own code path; the compiler eliminates dead branches.

### Cap slow random-input tests

If a sort can't handle large random inputs in reasonable time (e.g. anything cubic or worse) declare a cap so the correctness battery skips oversized arrays:

```rust
array_vis_bench::register_test_cap!("my slow sort", 1000);
```

## Adding a new gap sequence / rotation / etc.

These are not sorts — they're components consumed by sort families. Adding one:

1. Implement the trait in the appropriate `utils/` folder.
2. Annotate with `combo_codegen::component!(Role, MyType, "label")`.
3. Add a `register_*!` entry next to similar ones if you want the component visible as a standalone algorithm.

Every family that takes a matching slot will pick the new component up on the next build.
