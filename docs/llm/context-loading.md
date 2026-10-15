# Context loading

How to load enough context to work on this codebase without reading all of
it.

## Always read first

| file | why |
|---|---|
| `sort_logger/src/sort_logger.rs` | `SortLogger<T>`. Every algorithm body calls these methods and nothing else touches the array. |
| `sort_logger/src/sort_log.rs` | `SortLog<T>`, the event enum the renderer replays. |
| `array_vis_bench_traits/src/role/mod.rs` and the role files beside it | the role traits: `PartitionScheme` and its visitor, `PivotSelector`, `PivotInput`, `SmallSort`, `Rotation`, `QuickSelect`, `Auxiliary`, `Scratchspace`. |
| `array_vis_bench_traits/src/composable.rs` and `complexity.rs` | the annotation traits and the `Complexity` algebra the emit reads off each type. |
| `array_vis_bench_core/src/bench_registry.rs` | `AlgorithmEntry`, `Category`, `ALGORITHMS`, the per kind run drivers and batteries, the subprocess test dispatch. |
| `docs/writing-components.md` | the full account of how a Rust type, a `.spec` component and an `.avbs` query fit together. |

## Read for a specific area

| area | files |
|---|---|
| a family's algorithms | `crates/families/<family>/src/*.rs`, its `<family>.spec` and `<family>.avbs` |
| a slot filler | `crates/<axis>/<name>/src/lib.rs` and `<name>.spec` |
| the query language | `spec_core/src/avbs.pest` (grammar), `spec_core/src/avbs.rs` (lowering, wrappers) |
| the solver | `spec_core/src/solve.rs`, `enumerate.rs` (exhaustive and occurs check), `cardinality.rs` (the budget guard), `resolve.rs` |
| the catalog format | `spec_core/src/registry.rs` |
| what gets emitted | `spec_core/src/emit.rs`, `backend.rs`, `spec_emit/src/lib.rs` |
| the build | `spec_catalog/build.rs`, `spec_catalog/Cargo.toml`, `kinds.avbs`, `consumers.avbs` |
| golden counts and catalog tests | `spec_catalog/src/lib.rs` |
| consumers | `array_vis_bench_core/src/mains.rs`, `suites.rs`, `inputs.rs` |
| the interactive picker | `array_vis_bench_cli/src/lib.rs`, `sort_registry_core/src/lib.rs` |
| rendering | `sort_vis/src/mp4_bar_visualiser.rs` |
| performance | `src/bin/compare_sorts.rs`, `dump_asm.sh`, `asmdiff.sh`, the release profile in `Cargo.toml` |
| open work | `docs/todo.md`, `docs/implementation_plans/` |

Every folder has a README. Read it before reading the folder's sources.

## Skip

- `target/`, including the generated `catalog_entries.rs` (four megabytes).
  Read the `.avbs` query and the `.spec` fragment instead.
- `spec_core/registry.spec`: the compiler's own self test fixture, not the
  live catalog.
- `spec_catalog/quick_port.spec`: a retained baseline for one equivalence
  test, not a live fragment.
- `sort_logger/src/loggers.rs`: forty lines of trivial impls.
