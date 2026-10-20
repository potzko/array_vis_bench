# Architecture

The repo is organised as a compiler and the programs it compiles.

| part | role | lives in |
|---|---|---|
| the language | component declarations and the query grammar | `.spec` and `.avbs` files, `spec_core/src/avbs.pest` |
| the compiler | parse, solve, bound, emit | `spec_core` |
| the standard library | role trait impls, drivers, composers | `crates/*`, `array_vis_bench_traits` |
| the calling convention | `SortLogger`, the role traits, `AlgorithmEntry` | `sort_logger`, `array_vis_bench_traits`, `array_vis_bench_core` |
| the program | which compositions to build | each crate's `.avbs` plus `spec_catalog/kinds.avbs` |
| the link root | a binary that anchors a compiled catalog | `src/main.rs`, `array_vis_bench_min` |
| the consumers | visualise, check, benchmark | `array_vis_bench_core::mains`, `array_vis_bench_cli` |

## Crate graph

```
sort_logger
    |
array_vis_bench_traits ---------------------------+
    |                                             |
crates/{pivots,partitions,rotations,small_sorts,components}
    |                                             |
crates/families (drivers, composers)              |
    |                                             |
    +----> spec_catalog <---- array_vis_bench_core (the ABI) <---- sort_registry_core
               |  build.rs uses spec_core + spec_emit
               v
         array_vis_bench (src/main.rs)  ---- array_vis_bench_cli ---- sort_vis
```

Leaf crates depend only on the traits crate and the logger, so a consumer can
link two families without the rest of the workspace (`array_vis_bench_min`
proves it). `array_vis_bench_core` is algorithm free. `spec_core` is pure std
by default; the pest frontend is a feature used only at build time and in
tests.

## Build time flow

`spec_catalog/build.rs` runs once per build:

1. Gather. It runs `cargo metadata`, walks its own dependency closure and
   reads every crate's `[package.metadata.array_vis_bench]` block for a `spec`
   fragment and an optional `query` file. A crate that declares a fragment but
   is outside the closure produces a warning, because its family would
   otherwise vanish silently.
2. Merge. All fragments become one `spec_core::Registry`. Component names are
   a global namespace, so a duplicate is a build error. Parse order is
   deterministic and first wins where two components share a type head.
3. Solve. Each query is lowered independently (helper bindings stay scoped to
   their file) and solved against the merged registry. The cardinality pass
   bounds the output before anything is materialised, so an unbounded
   recursive role or an astronomical cross product fails fast with a count.
   A binding that solves to zero rows is an error, not a silent drop.
4. Emit. `spec_core::emit_entries` takes each ground type paired with the
   category its query wrapper assigned, and writes one `mod __entry_N` per type
   into `OUT_DIR/catalog_entries.rs`. The compiler emits only structure (the
   module, its `use` lines, the `Ty` alias) and asks the backend,
   `spec_emit::ArrayBackend`, for the registration body.
5. Count. `OUT_DIR/catalog_counts.rs` records one constant per query file and
   the total. The golden tests in `spec_catalog` assert each one.

The emitted body for a sort looks like this, with the complexity read off the
type rather than written in the catalog:

```rust
mod __entry_42 {
    use quick_sort_lib::{CanonicalQuickSort, Pivoted};
    use partition_lomuto::LeftLeftPartition;
    use pivot_first::FirstElement;
    use small_sort_basic::Size2SmallSort;
    pub type Ty = CanonicalQuickSort<Pivoted<LeftLeftPartition, FirstElement>, Size2SmallSort>;

    #[linkme::distributed_slice(ALGORITHMS)]
    static ENTRY: AlgorithmEntry = AlgorithmEntry {
        name: "quick sort<part: left-left<first>, small: size: 2>",
        category: Category::Sort,
        worst: <Ty as HasTimeBounds>::WORST,
        best: <Ty as HasTimeBounds>::BEST,
        average: <Ty as HasTimeBounds>::AVERAGE,
        space: <Ty as HasSpace>::SPACE,
        stable: <Ty as HasStability>::STABLE,
        adaptive: false,
        nondeterministic: false,
        max_input_size: None,
        run_with_input: run_default,
        run_correctness: run_correct,
    };
    #[ctor::ctor]
    fn register() { register_sort_variant(/* name, category path, facet axes */) }
}
```

Per category the backend picks the run driver (`run_sort_with_input`,
`run_rotation_with_input`, and so on) and the correctness battery. Kinds backed
by a role trait go through `CorrectnessSuite`, one impl per kind; full sorts
call `sort_battery` directly, passing the entry's cap, because a sort has no
role trait, only an inherent `sort` method.

## Startup flow

1. Link time. linkme collects every `#[distributed_slice(ALGORITHMS)]` static
   into one slice, and likewise the input slices.
2. Constructors. Each entry's `#[ctor]` registers its picker variant with
   `sort_registry_core::register_sort_variant`: a category path plus the
   `(role, value)` facet axes the picker navigates.
3. Validation. `bench_registry::validate_registries` runs before `main` and
   panics with a precise message on a duplicate name, a duplicate picker path,
   or a category without exactly one primary input.

## The consumers

`array_vis_bench_core::mains::Main` is the consumer abstraction: given a set
of `&AlgorithmEntry`, drive every one through a pipeline and return a
`MainReport`. It is the Rust counterpart of the query language's
`consumers: List<Mains<Set>>`.

| consumer | what it does |
|---|---|
| visualiser | runs the entry under `VisualizerLogger`, hands the event stream to `sort_vis::Mp4Visualizer`, which streams frames to `ffmpeg` |
| correctness | calls the entry's `run_correctness` pointer |
| benchmark | runs the entry under `NoOpLogger` against its category's primary input and times it |
| timing | the same, reported as a fastest and slowest table over the sort category |

The interactive binary is the visualiser consumer behind a picker.
`array_vis_bench_cli::run` reads the registry by name and function pointer
only, so it does not know or care how an entry was registered. The
`--consumers` flag runs the program in `spec_catalog/consumers.avbs` over the
whole emitted catalog instead.

### Inputs

Inputs are per category: `SORT_INPUTS`, `ROTATION_INPUTS`, `MERGE_INPUTS`,
`SMALL_SORT_INPUTS`, `QUICK_SELECT_INPUTS`, all in
`array_vis_bench_core/src/inputs.rs`. Exactly one entry per category is
primary and is what the harness uses when no input is named. Partition and
scratchspace reuse the sort inputs. The compiler selects algorithms, never
inputs.

### Correctness path

The workspace test suite walks `ALGORITHMS` and, for each entry, spawns the
test binary again with `AVB_RUN_CHECK_SORT=<name>`. A constructor in the child
sees the variable, calls the entry's `run_correctness` and exits before the
test runner starts. A freshly emitted algorithm is picked up with no separate
runner, and a hang or crash names the variant. Two per entry fields survive
emission: `max_input_size` (the cap, from `max_input` in the spec, passed on to
`sort_battery` so a bounded or slow sort skips longer patterns) and
`nondeterministic` (the trace varies between runs, so a check that compares
two traces of one entry skips it).

## Why the benchmark path costs nothing

- Every algorithm body is generic over `U: ?Sized + SortLogger<T>`. Under
  `NoOpLogger` each logger call is an empty inlined function. The registry's
  function pointers take `&mut dyn SortLogger<usize>`, but the dispatch happens
  once per run, outside the hot loop, through a monomorphic adapter the emit
  generates.
- Sorts expose an inherent `sort` rather than a trait object method, so there
  is no vtable in the inner loop.
- The release profile sets `codegen-units = 1` and thin LTO so LLVM sees every
  monomorphisation of a leaf method in one module. `.cargo/config.toml` builds
  with `target-cpu=native`.
- `src/bin/compare_sorts.rs` is the gate: generic quick, merge and heap sorts
  against hand rolled u64 baselines, each pair within 1.05x. `dump_asm.sh` and
  `asmdiff.sh` show the two assemblies side by side when the gate moves.

## Invariants

- One registration path. The spec catalog is the only emitter into
  `ALGORITHMS`. The legacy self registration features in a few family crates
  are off and enabled nowhere; validation would panic on the duplicate names.
- Inherit complexity, declare the rest. `worst`, `best`, `average`, `space`
  and `stable` come from the composable traits on the concrete type.
  `category` comes from the query wrapper; `adaptive`, `max_input`, `menu` and
  `nondeterministic` come from the catalog. No Big-O literal and no arithmetic
  in a spec file.
- Curate in the query, not in the code. A degenerate composition is removed
  with a set difference or a union in the `.avbs` file, never special cased in
  a driver.
- Deterministic emit order, so generated output and golden counts are stable
  across builds.
- One link unit. Implementations, program and consumer are linked together so
  the inlining story above holds. A publishable split is deferred until the
  conditions in `docs/todo.md` are met.
