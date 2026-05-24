# Patterns and Conventions

Follow these when writing or modifying code in this project.

## Sort implementation patterns

### All data access goes through the logger

The single most important rule. Every comparison and write must call a `SortLogger` method:

```rust
// CORRECT:
logger.cond_swap_gt(arr, i, j);
logger.write_data(arr, i, value);
logger.cmp_le_accross(arr_a, i, arr_b, j);

// WRONG — works for benchmarks but produces no visualisation events:
if arr[i] > arr[j] { arr.swap(i, j); }
arr[i] = value;
```

Reading is fine without the logger — only writes and comparisons need to be observed:

```rust
let tmp = arr[i];                       // OK
logger.write_data(arr, i, arr[j]);      // log the write
```

### Auxiliary arrays go through the logger

```rust
let mut buf = logger.create_aux_arr_t(len);
logger.write_accross(arr, i, &mut buf, j);
logger.free_aux_arr_t(&buf);
```

This lets the visualiser draw aux buffers next to the main array.

### Generic over `<T: Ord + Copy, U: ?Sized + SortLogger<T>>`

```rust
fn my_sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) {
```

`U: ?Sized` lets the function take both `&mut NoOpLogger` (bench, fully inlined) and `&mut dyn SortLogger<T>` (visualiser). Only drop `?Sized` if you call the trait's `where Self: Sized` cross-`T` helpers.

### Const-generic booleans for flags

```rust
pub struct MySort<S: SmallSort, const EARLY_EXIT: bool> { ... }
```

Each flag combination monomorphises to its own code path; the compiler eliminates dead branches.

## Registration patterns

### Single-leaf or self-contained sort: `sort_registry_macro::sort_family!`

```rust
sort_registry_macro::sort_family! {
    type Sort = MySort;
    name        = "my sort";
    big_o       = "O(N^2)";
    stable      = true;
    direct_sort = true;
    path        = ["my sorts", "my sort"];
}
```

### Cross-product family: `combo_codegen::family!`

Use this when slots draw from project-wide `component!`-annotated types (small-sort, rotation, pivot selector, etc.). See `src/sorts/quick_sorts/quick_sort.rs` and `src/sorts/merge_sorts/top_down.rs` for real examples.

### Standalone component (rotation, partition, merge, quick-select, small-sort)

Use the matching `register_*!` macro alongside the implementation. Each emits one `bench_registry::AlgorithmEntry` with the right `Category::*`.

### Never edit a list of algorithm names

Every registration mechanism lands in `bench_registry::ALGORITHMS` (a `linkme` distributed-slice) automatically. If you find yourself editing a list of algorithm names somewhere central, you're doing it wrong.

## Naming conventions

- Sort families: `snake_case` folder names (`merge_sorts/`, `shell_sorts/`).
- Algorithm display names: lowercase with spaces (`"merge sort top-down pp ee"`).
- Strategy types: `PascalCase` (`ReversalRotation`, `InsertionSmallSort`).
- Strategy display labels (inside `component!` / `sort_family!`): lowercase with hyphens (`"reversal"`, `"ins-32"`).
- Nested loop variables: `i, ii, iii, iv, ...` (not `i, j, k, m, ...`).

## File organisation

- Each sort family lives in its own folder under `src/sorts/`.
- Each folder has a `mod.rs` for module declarations and (where used) `include!` of the generated combinations file.
- Strategy traits live in `src/utils/` (rotation, gap sequences, branching, small_sort) or next to their primary consumer (`quick_sorts/partitions.rs`, `quick_sorts/pivot_selectors.rs`).
- Every folder has a `README.md` — see the documentation maintenance guide.

## What NOT to do

- **Don't maintain a central list of algorithm names.** Registration is automatic.
- **Don't use `arr.swap(i, j)` or direct assignment in sorts.** Always go through the logger.
- **Don't put sort-specific code in `utils/`.** Utils are shared building blocks.
- **Don't create `dyn SortLogger<T>` in the benchmark hot path.** Bench code uses monomorphic function pointers; a trait object would defeat inlining.
- **Don't patch one bad combination in a cross-product.** Drop the problem axis instead (the project's coding-preferences rule: easier to maintain, faster to converge).
