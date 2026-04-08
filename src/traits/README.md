# traits

Core trait definitions, global registries, and the `create_sort!` macro. This is the glue layer between sort implementations, benchmarks, and the visualiser.

## Traits

### `SortAlgo<T, U>`
The trait every sort implements. Four associated items:
- `name()` — human-readable name (e.g. `"bubble sort"`).
- `big_o()` — time complexity string (e.g. `"O(N^2)"`).
- `stable()` — whether the sort preserves equal-element order.
- `sort(arr, logger)` — the sort itself.

### `SortLogger<T>`
Re-exported from the `sort_logger` crate. See [sort_logger/README.md](../../sort_logger/README.md).

## Registries

Two global `HashMap`s hold function pointers, populated at startup by `#[ctor]` hooks:

| Registry | Value type | Purpose |
|---|---|---|
| `SORT_REGISTRY` | `fn(&mut [usize], &mut NoOpLogger)` | Benchmark dispatch. Monomorphic, fully inlinable — no trait objects. |
| `SORT_VIS_REGISTRY` | `fn(&mut [usize], &mut dyn SortLogger<usize>)` | Visualisation dispatch. Accepts a dyn logger for recording. |

## `create_sort!` macro

Reduces sort registration to a one-liner:
```rust
create_sort!(bubble_sort, "bubble sort", "O(N^2)", true);
```
This generates: a `SortImp<T, U>` implementing `SortAlgo`, a monomorphic `SortReg` with `#[derive(SortRegistry)]`, and a `linkme` distributed-slice entry for benchmark collection.

## Files

- `sort_traits.rs` — `SortAlgo<T, U>` definition.
- `log_traits.rs` — re-exports from `sort_logger`.
- `mod.rs` — registries, `create_sort!`, and helper functions.
