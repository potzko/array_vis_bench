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

## Findings from porting the real sort shapes

Porting QuickSort / merge / shell / insertion against faithful stubs (real
module paths + real signatures, incl. rustc-enforced arity) surfaced four issues:

1. **Imports were entirely unmodeled.** Real families carry `uses = [...]`. Added
   a `uses` directive, unioned from every nested child during `resolve`. Gotcha:
   emitting `use X;` at crate scope for two sorts that share an import is a
   *duplicate-import compile error* — so `emit_one` now wraps each sort in its
   own private module and re-exports the public names. (Alternative that kills
   the whole class: put fully-qualified paths in the `type` templates and drop
   `uses`.)

2. **Const generics are richer than "positional usize".** Real sorts have
   `const PING_PONG: bool`, `const EARLY_EXIT: bool`, and type+const components
   (`InsertionSmallSort<S, const N>`). Generalized consts to string literals
   (int **or** `true`/`false`), bindable by name (`ping_pong = true`) or
   positionally (`insertion<32>`).

3. **THE headline — flat vs nested arity is a real fork, and my earlier "it's
   just a catalog edit" was wrong.** Real `QuickSort<P, V, SS>` keeps the pivot
   `V` as a *sibling* of the partition `P`. Two consequences:
   - The **nested** DSL we agreed on (`partition = LL< pivot = … >`) cannot emit
     the real sibling type under the current local-substitution engine — the
     pivot would get embedded into the partition's type, but real
     `LeftLeftPartition` is a unit struct. Honoring nesting needs *either* a code
     refactor (make partitions generic over pivot) *or* an engine hoist/project
     mechanism. It is **not** a pure catalog edit.
   - The **flat** layout (used here, matches the real type) emits fine, but the
     per-slot role check **cannot express the cross-slot arity constraint**
     (single partition ⇒ single pivot). So `resolve` *accepts*
     `quick_sort<partition=LL, pivot=ninther_dual>` and only **rustc** rejects it
     (`<NintherDualPivot as PivotInput>::Arity = One` unsatisfied). Proven by
     `flat_quicksort_accepts_arity_mismatch_at_registry_level` + a manual
     compile-fail check.

4. **Flat enumeration overproduces.** `enumerate("Sort")` yields 48 variants, 42
   of them `quick_sort` — and ~half are arity-illegal, so they can't be compiled.
   build.rs has to *drop the whole quick_sort family* before emitting (see its
   `cargo:warning`). The real codebase sidesteps this by **splitting into
   separate family templates** (a single-pivot family and a dual-pivot family
   with `DualPivotPartition` hardcoded), each internally arity-consistent.

### Decision points this raises

- **QuickSort layout:** (a) template-split single/dual like the real code does
  (matches reality, duplicates the small-sort axis); (b) nest + make partitions
  generic over pivot (ergonomic, arity local, needs a code refactor); (c) add a
  cross-slot constraint to the registry (`require partition.Arity == pivot.Arity`);
  or (d) add a hoist/projection so a nested DSL can emit sibling type args.
- **Imports:** fully-qualified paths in templates (no `uses`, no scoping hazard)
  vs. `uses` + per-sort module scoping (shorter templates).

## Known prototype simplifications

- The registry is `include_str!`'d into the macro/generator; in production it's
  `Cargo.toml` metadata read in build.rs.
- The macro round-trips through `to_string()`, so its error spans are coarse.
  A production macro would keep `syn` spans for precise diagnostics.
- Const slots (`insertion_sort<N>`) aren't enumerated — the enumerator uses the
  registry default. A real generator would take an explicit value set per const.
