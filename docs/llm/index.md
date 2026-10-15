# LLM guide

You are working on `array_vis_bench`, a Rust workspace that composes, visualises,
checks and benchmarks algorithms over ordered data. This folder is for
assistants. Read in order:

1. [context-loading.md](context-loading.md): which files define the
   abstractions, and which to skip.
2. [patterns.md](patterns.md): the conventions every change follows.
3. [documentation-maintenance.md](documentation-maintenance.md): what to
   update when the code changes, and the house style the prose follows.

## Orientation

- About sixty crates. Leaves implement role traits, family crates hold
  drivers and composers, `spec_catalog` compiles every `.spec` and `.avbs`
  file in its dependency closure into the registry, and the binaries are link
  roots over that registry.
- Every algorithm is generic over a `SortLogger`. Read
  `sort_logger/src/sort_logger.rs` before touching any algorithm.
- Nothing is registered by hand. A variant exists because a query solved to
  it. If you find yourself editing a list of names, stop.
- Counts are golden. `spec_catalog/src/lib.rs` pins every family's emitted
  count; a change in the catalog moves them, and you update them only after
  checking the new variants are the intended ones.
- The human docs in `docs/` and the file top overviews in every crate are
  part of the deliverable. Keep both in step with the code.
