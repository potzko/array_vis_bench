# array_vis_bench

A Rust workspace for benchmarking and visualising sorting algorithms and
their building blocks (rotations, partitions, merges, quick-selects,
small-sorts).

Algorithms are not hand-listed. Each leaf crate declares its families in
`Cargo.toml` metadata; a build-time scanner expands every cross-product
of generic parameters into one entry in a single global registry. The
interactive visualiser, speed-test binary, Criterion benches, and
correctness suite all iterate that one registry.

Current catalog (as of this README):

```
sorts          4351 variants
small-sorts       7 variants
rotations        11 variants
merges           24 variants
partitions       30 variants
quick selects    56 variants
```

## Binaries

- `array_vis_bench` (default `cargo run --release`)
  Interactive visualiser. Walks a category -> family -> variant menu,
  asks for an input shape and array size, then renders an MP4 of the
  resulting `SortLog` stream to `output.mp4`.

- `speed_test` (`cargo run --release --bin speed_test`)
  Runs every registered algorithm against the primary input for its
  category and prints median wall-clock times, fastest-first.

- `compare_sorts` (`cargo run --release --bin compare_sorts`)
  Performance regression gate. Pits the workspace's generic quick /
  merge / heap sorts against hand-rolled u64 baselines and asserts each
  pair stays within a 1.05x margin.

- `measure_random_shell` (`cargo run --release --bin measure_random_shell`)
  One-off complexity probe for the random-shell-sort variants.

- Criterion benches: `cargo bench --bench sorts`. Adaptive thresholding
  keeps slow algorithms from dominating wall-time.

`cargo test` runs the full workspace correctness suite (~4500 sort
variants plus per-crate unit tests).

## Categories and menu structure

Every registered algorithm belongs to one of six categories
(`bench_registry::Category`). The top-level menu groups by category,
then drills down a path defined per family (e.g.
`sorts > quick sorts > dual pivot > combined > median of 3 / median of 3 > insertion: 32`).

```
sorts          comb / merge / quick / bubble / heap / insertion /
               quick heap / fun / circle / cycle / rod / shell
small-sorts    size-1, size-2, insertion (linear / binary), network 16
rotations      reversal, juggling, trinity, drill, helix, piston,
               grail, gries-mills, bridge, contrev, auxiliary
merges         standalone merge variants (top-down, bottom-up, ping-pong,
               rotation-based, ...)
partitions     standalone partition variants (Lomuto, Hoare, block,
               three-way, moving-pivot, dual-pivot, ...)
quick selects  single- and dual-pivot quickselect over the partition
               and pivot-selector axes
```

Inputs are per-category: `SORT_INPUTS`, `ROTATION_INPUTS`,
`PARTITION_INPUTS`, etc. Each input registry has exactly one entry
marked `primary: true` that the harness picks by default.

## Workspace layout

```
.                        binary crate (CLI entry point + src/bin/*)
array_vis_bench_core     ALGORITHMS distributed-slice, RunConfig, input
                         registries, correctness battery
array_vis_bench_full     wiring crate: re-exports every leaf family and
                         includes the generated combinations files
array_vis_bench_min      minimal wiring variant for fast iteration
array_vis_bench_traits   role traits (PartitionScheme, PivotInput,
                         SmallSort, Rotation, ...) and the composable
                         annotation traits (HasTimeBounds, HasSpace,
                         HasStability, PivotQuality)
sort_logger              SortLog<T> event enum, SortLogger<T> trait,
                         NoOpLogger, VisualizerLogger
sort_vis                 MP4 renderer (replays a Vec<SortLog> stream)
sort_registry_core       navigation registry + faceted picker support
sort_registry_macro      sort_family! proc-macro that expands one family
                         metadata block into N AlgorithmEntry leaves
combo_codegen            build-script scanner: reads Cargo.toml
                         metadata, expands components into family
                         cross-products, writes one combinations file
                         per family under OUT_DIR

crates/families/*        one crate per sort family (quick_sort_lib,
                         heap_sort_lib, merge_sort_lib, ...)
crates/partitions/*      one crate per partition primitive
crates/pivots/*          one crate per pivot selector
crates/rotations/*       one crate per array rotation algorithm
crates/small_sorts/*     small-sort strategy crates
crates/components/*      shared component types (comb ratios, gap
                         distributions, heap internals, ...)
crates/registries/*      standalone-category registries that pull
                         primitives into the menu directly

docs/                    architecture, registration, trait system,
                         adding-a-sort walkthrough, llm/ subfolder
```

## How registration works

There is no central list. Every algorithm declares itself in one of two
ways and ends up as a single `AlgorithmEntry` in
`array_vis_bench_core::bench_registry::ALGORITHMS` (a `linkme`
distributed-slice).

1. Metadata-driven (the common path).
   Each family crate writes its component types and cross-products as
   `[[package.metadata.array_vis_bench.components]]` and
   `[[package.metadata.array_vis_bench.families]]` entries in
   `Cargo.toml`. At build time, `combo_codegen` scans the workspace,
   resolves roles, expands every cross-product, and writes a generated
   `<family>_combinations.rs` to `OUT_DIR`. Each generated file consists
   of `sort_family! { ... }` invocations; the proc-macro expands those
   into per-variant `AlgorithmEntry` statics registered via `#[ctor]`.

2. Hand-rolled registration macros (`register_rotation!`,
   `register_partition!`, `register_merge!`, ...) for primitives that
   don't form clean cross-products. Same end state: one
   `AlgorithmEntry` per leaf, same distributed-slice.

Adding a new component, pivot selector, or partition usually means:
create the crate, declare its role + uses in its `Cargo.toml`, list it
as a workspace member. No edits to anything else; the build script
re-derives the cross-products and the new variants appear in the menu
the next time you `cargo run`.

See `docs/architecture.md`, `docs/registration.md`,
`docs/adding-a-sort.md`, and `docs/trait-system.md` for the details.

## Logging and visualisation

`SortLogger<T>` is the central abstraction every sort routine writes to.
Production runs use `NoOpLogger` (zero-cost) and benchmarks compile
through it without overhead. The visualiser binary swaps in
`VisualizerLogger<T>`, captures the full event stream as `Vec<SortLog<T>>`,
and hands it to `sort_vis::Mp4BarVisualiser` to render. Auxiliary array
events (`log_aux_arr_u`, `log_aux_arr_u8`, ...) let sorts allocate /
free / write side-buffers that the renderer can display alongside the
primary array.

## Output

Visualiser runs write `output.mp4` to the working directory. Frame rate,
resolution, and a few render knobs live near the top of `src/main.rs`.

## Note for LLM assistants

`docs/llm/` contains a context-loading guide, key files to always index,
patterns to follow, and instructions for keeping documentation up to
date. Read those first before making changes.
