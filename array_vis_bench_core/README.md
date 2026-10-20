# `array_vis_bench_core`

The ABI every emitted entry links against and every consumer reads. It is
algorithm free: leaves register into the distributed slices declared here.

- `src/bench_registry.rs`: `AlgorithmEntry`, `Category`, `ALGORITHMS`, the input slices, the per kind run drivers and correctness batteries, and the subprocess test dispatch.
- `src/suites.rs`: `CorrectnessSuite`, one impl per role trait backed kind.
- `src/mains.rs`: `Main`, the consumer abstraction, with the visualiser, correctness and benchmark consumers.
- `src/navigation.rs`: the picker's view of `ALGORITHMS`, built from each entry's `menu`, `axes` and `label_template`.
- `src/inputs.rs`: the registered input shapes per category.
- `src/visualise.rs`: find an entry by name and run it under a logger.
- `src/array_gen.rs`: deterministic array generators.
