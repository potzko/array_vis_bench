# `spec_emit`

The array domain emit backend. Implements `spec_core::EmitBackend` and
writes, for every solved type, the `AlgorithmEntry` registration body for
its category: the run adapter, the correctness adapter, the inherited
complexity reads and the picker registration.

- `src/lib.rs`: `ArrayBackend`, `AbiPaths`, and one entry body per category.
