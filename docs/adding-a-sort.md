# Adding a New Sort

How to add a new sorting algorithm to the project.

## Option 1: Standalone sort (simplest)

For a one-off sort that doesn't need parameterisation.

### 1. Create the file

Add a file in the appropriate family folder (e.g. `src/sorts/bubble_sorts/my_sort.rs`):

```rust
use crate::traits::log_traits::SortLogger;

fn my_sort<T: Ord + Copy, U: SortLogger<T>>(arr: &mut [T], logger: &mut U) {
    // Your algorithm here.
    // Use logger methods for ALL data access:
    //   logger.cond_swap_gt(arr, i, j)  — compare and swap
    //   logger.write_data(arr, i, val)  — write a value
    //   logger.cmp_lt(arr, i, j)        — compare without swap
    //
    // Do NOT use arr[i] = arr[j] directly — it won't be logged.
}

create_sort!(my_sort, "my sort", "O(N^2)", true);
```

### 2. Add the module

In the family's `mod.rs`:

```rust
pub mod my_sort;
```

That's it. The `create_sort!` macro handles everything: `SortAlgo` impl, `BENCH_SORTS` linkme entry, `SORT_REGISTRY` insertion, and `sort_registry_core` metadata. The sort will appear in benchmarks and (once the family is wired into `sorts/mod.rs`) the visualiser.

## Option 2: Parameterised sort family

For sorts with pluggable strategies (like shell sort + gap sequences, or merge sort + rotation algorithms).

### 1. Define your strategy trait

```rust
pub trait MyStrategy {
    const NAME: &'static str;
    fn do_thing(/* ... */);
}
```

### 2. Implement the sort generically

```rust
pub struct MySort<S: MyStrategy> { _phantom: PhantomData<S> }

impl<S: MyStrategy> MySort<S> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
        // Use S::do_thing() for the parameterised part
    }
}
```

### 3. Register combinations

**Option A: `sort_family!` macro** (preferred for many combinations):

```rust
sort_family! {
    type Sort = MySort<{Strategy}>;

    Strategy {
        StrategyA => "a"
        StrategyB => "b"
    }

    name   = "my sort {Strategy}";
    big_o  = "O(N log N)";
    stable = false;
    path   = ["my sorts", "{Strategy}"];
}
```

**Option B: Manual distributed slice** (for custom registration logic):

```rust
struct MyEntry {
    name: &'static str,
    sort_fn: SortFn,
    // ...
}

static MY_ENTRIES: &[MyEntry] = &[
    MyEntry { name: "my sort a", sort_fn: sort_a, ... },
    MyEntry { name: "my sort b", sort_fn: sort_b, ... },
];

#[ctor::ctor]
fn register() {
    for entry in MY_ENTRIES {
        SORT_REGISTRY.lock().unwrap().insert(entry.name.to_string(), entry.sort_fn);
        sort_registry_core::register_sort_path(entry.name, "O(N log N)", false, &["my sorts", entry.name]);
    }
}
```

## Rules for sort implementations

### Always use the logger

Every comparison and data movement must go through `SortLogger` methods. Direct array access (`arr[i] = arr[j]`) works for benchmarks but produces no visualisation data.

Key methods:
- `logger.cond_swap_lt(arr, i, j)` — swap if `arr[i] < arr[j]`, return whether swapped.
- `logger.cond_swap_gt(arr, i, j)` — swap if `arr[i] > arr[j]`.
- `logger.write_data(arr, i, value)` — write `value` to `arr[i]`.
- `logger.write_accross(src, i, dst, j)` — copy `src[i]` to `dst[j]`.
- `logger.cmp_le_accross(a, i, b, j)` — compare elements across two arrays.
- `logger.create_aux_arr_t(len)` / `logger.free_aux_arr_t(&buf)` — allocate/free auxiliary arrays.

### Reading is free

Reading `arr[i]` to get a value (e.g. for a temporary variable) does not need to go through the logger — only *writes* and *comparisons* need logging. This is fine:

```rust
let tmp = arr[i];  // reading: no logger needed
logger.write_data(arr, i, arr[j]);  // writing: must use logger
logger.write_data(arr, j, tmp);     // writing: must use logger
```

### Test your sort

The `sort_test` module can validate correctness. Add tests in your file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::log_traits::NoOpLogger;

    #[test]
    fn basic() {
        let mut arr = vec![5, 3, 1, 4, 2];
        my_sort(&mut arr, &mut NoOpLogger);
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
    }
}
```
