# `src`

Source of `spec_core`. The crate README one level up says what it is for.

- `avbs.pest`: AVBS, the query language (the surface the user authors).
- `avbs.rs`: The AVBS query language, pest frontend.
- `backend.rs`: The domain seam for entry emission.
- `cardinality.rs`: Stage 0'', STATIC SIZE ANALYSIS. Before `solve` materializes anything, this pass computes an *upper bound* on how much the query will produce, so...
- `emit.rs`: Stage 4, emit Rust source from resolved specs.
- `enumerate.rs`: Stage 0 (mode 2 only), a program produces the spec trees.
- `lib.rs`: The shared engine: registry text -> catalog, spec text -> tree, tree + catalog -> resolved concrete type, and Rust emission.
- `registry.rs`: Stage 1, the text catalog: parse `registry.spec` into a component graph.
- `resolve.rs`: Stage 3, resolve a `SpecNode` against the `Registry`: fill defaults, check slot roles/arity, collect imports, and produce a concrete Rust type...
- `solve.rs`: Stage 0', the typed constraint SOLVER. Lowers a `Query` (let-bindings, shared variables, refinements, and holes with quantifiers) into a *set* of...
- `spec.rs`: Stage 2, parse ONE spec tree (`Alias = quick_sort< ... >`) into a `SpecNode`.
