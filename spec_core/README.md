# `spec_core`

The constraint compiler. Pure std by default; the pest frontend is a feature
used at build time and in tests.

- `src/registry.rs`: parse `.spec` fragments into the component graph.
- `src/avbs.pest` and `src/avbs.rs`: the query grammar and its lowering onto the solver's query form, including the emit wrappers.
- `src/spec.rs`: the spec tree and query types.
- `src/solve.rs`: the solver: shared variables, refinements, set difference, unions, captures, sampling.
- `src/enumerate.rs`: exhaustive enumeration of a role, and the occurs check behind `finite`.
- `src/cardinality.rs`: the static size bound that refuses an unbounded or oversized query before enumeration.
- `src/resolve.rs`: resolve a tree to a concrete type expression, label and facet axes.
- `src/emit.rs` and `src/backend.rs`: the structural emit and the backend seam a domain fills.
- `registry.spec`: the compiler's own example catalog for its unit tests.
