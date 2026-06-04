# spec-tree compiler — prototype

Realizes the pipeline:

```
registry.spec (text)  ──►  spec_core  ──►  Rust
  catalog of types          parse → resolve → emit
  & algorithms              (defaults + role/arity checks)
                                 │
                ┌────────────────┴─────────────────┐
        spec_macro (mode 1)               generator (mode 2)
        ONE tree, inline                  a PROGRAM enumerates many trees
```

No cross-product baked into the macro, no head-count rule. Legality is encoded
as **roles** in the registry (exact grammar), with rustc as the ultimate
backstop. Recursion in the enumerator terminates on **literal tree depth** — an
honest knob — not a per-head visit heuristic.

## Run it

```
cd spec_compiler_proto
cargo test                 # spec_core engine tests + demo round-trip tests
cargo run -p generator     # mode 2: print the generated dispatch table to stdout
```

Detached workspace — deliberately **not** a member of the parent
`array_vis_bench` workspace, so it can't affect the main build.

## Crates

| Crate | Role |
|---|---|
| `registry.spec` | the text catalog — name → type template, label, `provides` roles, slots, defaults. The shared **specification** contract. |
| `spec_core` | pure-std engine: `Registry::parse`, `parse_spec`, `resolve` (defaults + role/arity), `emit_one`, `enumerate`, `generate_table`. No syn/quote. |
| `spec_macro` | mode 1 — inline `sort_spec!(Alias = …)`; a thin shell over `spec_core`. |
| `generator` | mode 2 — loads the registry, enumerates legal sorts to a depth bound, prints Rust. |
| `demo` | stub types + both front-ends in use; `build.rs` runs the engine to emit `generated::SORTS`. |

## The compile stages ("more than one compile step")

1. **Registry load** — `Registry::parse` turns the text catalog into a component graph.
2. **Spec parse** — one tree (`parse_spec`) or many (`enumerate`).
3. **Resolve** — name resolution, default filling, role/arity checks → concrete type string + label.
4. **Emit** — Rust source (inline tokens via the macro, or a `.rs` file via build.rs / the generator).
5. **rustc** — full type checking + monomorphization. The final, authoritative legality gate.

Mode 2 prepends stage 0: a program *produces* the spec trees. That's where any
bounded-enumeration policy lives — demoted from a codegen invariant to a
generator concern, and entirely skippable when you hand-write specs.

## Nesting payoff (proven)

Pivot is nested under the partition, and the slot role encodes arity:
`LeftLeftPartition<V: SinglePivot>` cannot take a `DualPivot` selector.

- `spec_core::tests::arity_violation_is_rejected_by_the_engine` — the engine
  rejects `LL_partition<pivot=tukey_dual>` with a role error, *before* rustc.
- `enumerates_only_legal_combos` — enumeration yields 14 sorts; the dual
  selector appears only with the dual-pivot partition, never with LL/LR.
- rustc is the backstop: the bad composition also fails to compile.

## Known prototype simplifications

- The registry is `include_str!`'d into the macro/generator; in production it's
  `Cargo.toml` metadata read in build.rs.
- The macro round-trips through `to_string()`, so its error spans are coarse.
  A production macro would keep `syn` spans for precise diagnostics.
- Const slots (`insertion_sort<N>`) aren't enumerated — the enumerator uses the
  registry default. A real generator would take an explicit value set per const.
