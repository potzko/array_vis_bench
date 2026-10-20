# `spec_emit`

The array domain emit backend. Implements `spec_core::EmitBackend` and
writes, for every solved type, the `AlgorithmEntry` registration body for
its category: the run adapter, the correctness suite call, the inherited
complexity reads and the picker registration.

- `src/lib.rs`: `ArrayBackend`, `AbiPaths`, the `KINDS` table with one row per kind, and the one entry template that reads a row.
