# array_vis_bench

The root crate and entry point. Wires together all the sub-crates, owns the sort implementations, and drives two binaries:

- **`array_vis_bench`** — interactive visualiser: prompts for a sort and array size, runs the sort with a `VisualizerLogger`, and writes an animated GIF showing the sort in action.
- **`bench`** — benchmark runner: iterates every entry in `BENCH_SORTS` and reports timings.

## Structure

```
src/
  sorts/          — all sort algorithm implementations
    shell_sorts/  — shell sort + shell-shell sort variants, self-registering via linkme
    insertion_sorts/
    ...
  utils/          — shared algorithmic helpers (gap sequences, branching strategies, array gen)
  traits/         — SORT_REGISTRY, SortFn type, create_sort! macro, re-exports from sort_logger
  visualise/      — GIF rendering pipeline (delegates to sort_vis)
  main.rs         — CLI, sort dispatch, startup validation
```

## Sort self-registration

Every sort registers itself at link time using `linkme` distributed slices and `ctor` startup hooks — `main.rs` never needs a hardcoded list of sorts. Adding a sort means calling one macro (`register_sequence!` or `register_branching!`) and nothing else changes.
