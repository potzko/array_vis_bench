# sort_logger

Instrumentation layer for sort algorithms. This crate is the only dependency that every sort implementation needs — it intentionally has no dependencies of its own.

## What it contains

- **`SortLog<T>`** — an enum of every observable event a sort can emit: comparisons, swaps, writes, auxiliary array allocations/frees, and free-form marks.
- **`SortLogger<T>`** — the trait all sort algorithms are generic over. Provides default-implemented helpers (`cond_swap_lt`, `write_data`, `cmp_gt_accross`, etc.) built on top of a single required method: `fn log(&mut self, SortLog<T>)`.
- **`NoOpLogger`** — zero-cost logger for benchmarking. All `log` calls compile away entirely.
- **`VisualizerLogger<T>`** — accumulates every `SortLog` into a `Vec` for later playback by the visualiser.
- **`arr_name!`** — macro that turns a slice reference into a stable identity (`ptr as usize`) so the renderer can track which array is which.

## Why it's a separate crate

Keeping instrumentation isolated means sort implementations can live in their own crates with a single lightweight dependency. The visualiser and benchmarker depend on this crate too, but they don't pull in sort code — the dependency graph stays acyclic and clean.

## dyn-compatibility

`SortLogger<T>` is dyn-compatible. Methods that are generic over a second type `U` (e.g. `swap<U>`, `cond_swap_le<U>`) are gated `where Self: Sized` and excluded from the vtable. All T-specific methods (`cond_swap_lt`, `cond_swap_gt`, etc.) are inlined and work on `dyn SortLogger<T>` without allocating.
