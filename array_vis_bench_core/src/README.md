# `src`

Source of `array_vis_bench_core`. The crate README one level up says what it is for.

- `array_gen.rs`: Tiny array generators used by the input registry.
- `bench_registry.rs`: Algorithm + input registry, the runtime spine of the workspace.
- `inputs.rs`: Canonical inputs for every algorithm category.
- `lib.rs`: Registry, harness, inputs, and visualiser shell for `array_vis_bench`.
- `mains.rs`: `Main`, the consumer abstraction.
- `navigation.rs`: The picker's view of `ALGORITHMS`, built on demand from each entry's `menu`, `axes` and `label_template`.
- `suites.rs`: `CorrectnessSuite`, a correctness battery as a per-KIND construct.
- `visualise.rs`: Run-and-render orchestration.
