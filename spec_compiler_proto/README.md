# spec-tree compiler — prototype

A small **typed constraint language** for generating sort types. Pinned specs,
partial families, and full generation are the **same evaluator** with different
numbers of holes — and a structurally-shared variable makes an arity-mismatched
sort *unrepresentable at generation time*, not merely caught later by rustc.

```
registry.spec (text)  ──►  spec_core  ──►  Rust
  catalog of types          parse → solve → resolve → emit
  & roles & projected            │
  params                         │   holes are the ONLY difference between
                ┌────────────────┴───┐   pinned / partial / full
        spec_macro (mode 1)     generator + build.rs (mode 2)
        ONE tree, inline        a QUERY lowers to a SET of trees
```

## Two-layer validity (the whole design in one sentence)

The registry proves only **structural validity** (roles, arity, equality
constraints); **rustc** is the final proof of Rust validity. We generate only
what the registry can prove, so rustc never actually rejects anything — it is a
redundant backstop, never the first line of defence.

## Run it

```
cd spec_compiler_proto
cargo test                              # engine + demo round-trip tests
cargo run -p generator                  # solve the default arity-safe family → Rust
cargo run -p generator -- 'let s: Sort = .;'   # solve any query
```

Detached workspace — deliberately **not** a member of the parent
`array_vis_bench` workspace, so it can't affect the main build.

## The constraint language

A query is a sequence of `let`-bindings ending in a target (the last binding).
The `let` keyword is optional (`p: Pivot = first_element` works too).

```
depth 3;                                       # optional: bound auto-expansion
3 of                                           # optional: N distinct WHOLE sorts
let p: Pivot = .;                              # a role-typed HOLE
let part: Partition[pivot = p] = .;            # a hole REFINED by a shared var
let s: Sort = quick_sort(                      # the TARGET
    partition = part,                          # references (shared variables)
    pivot     = p,
    small_sort = .,
);
```

### Values

| Form | Meaning |
|---|---|
| `quick_sort(a = …, b = …)` | component application (named or positional args) |
| `first_element` | a bare ident: a **shared variable** if bound earlier, else a nullary component |
| `.` / `*` | a **hole**, exhaustive — every valid filler (a cross-product dimension) |
| `?` | a hole, **one** random filler |
| `?N@seed` | a hole, **N distinct** random fillers, seeded (`?3@42`) |
| `32`, `true` | a const literal |

### Hole quantifiers — cross-cutting rules

- **Dedup** is on the *resolved canonical sort* (the elaborated Rust type), so two
  spec trees that mean the same type are one sort.
- **All randomness is seeded** (`@seed`, default 0) → reproducible builds.
- **`N` larger than the population clamps** to the population and emits a
  `cargo:warning` — never a silent truncation.
- **Recursion is bounded** by the per-query `depth` knob (default `4`); a hole of
  a recursive role expands only down to that depth.

### Shared variables → arity made structural (the headline)

`Partition[pivot = p]` refines a partition on its **projected** `pivot` param.
Partitions are unit structs — they don't take a pivot type argument — so `pivot`
is declared `project pivot PivotSingle` / `project pivot PivotDual` in the
registry: a **structural-only** param that never appears in the emitted type and
exists purely so a refinement can thread a role onto the shared variable.

Because `p` is the *same* variable in the partition refinement and the
`quick_sort` pivot slot, the solver threads one value through both and checks
role membership:

- `part = LeftLeftPartition` ⇒ `p` must provide `PivotSingle`
- `part = DualPivotPartition` ⇒ `p` must provide `PivotDual`

So `quick_sort<partition = LL, pivot = ninther_dual>` is **never built** — not
"produced then rejected by rustc". A unit partition that declares *no*
`project pivot` cannot satisfy `Partition[pivot = …]` at all; that exclusion *is*
the arity filter. (See `spec_core::tests::shared_pivot_makes_arity_mismatch_unrepresentable`:
21 arity-correct quick sorts, the bad combo absent.)

### "Slight dependent typing": numbers as shared, defaultable, enumerable values

Const generics (`InsertionSmallSort<S, const N>`, a d-ary `HeapSort<const K>`)
are first-class — *within the discipline*, which means **structural** use of
numbers only, never reasoning *about* them:

```
heap_sort()                # → HeapSort<2>      (the declared default)
heap_sort(arity = 4)       # → HeapSort<4>      (explicit)
heap_sort(arity = *)       # → HeapSort<2|3|4>  (enumerate the declared value set)
heap_sort(arity = ?1@5)    # → one, sampled from the set, seeded

let flag: Flag = true;     # a shared CONST variable…
top_down_merge(small_sort = no_small_sort, ping_pong = flag, early_exit = flag)
                           # → TopDownMergeSort<NoSmallSort, true, true>
                           #   pp == ee BY CONSTRUCTION — structural equality
```

A const declares its "neat values" once in the registry
(`const arity 2 values 2 3 4`); a quantified const hole ranges over that set by
**membership only**. What stays out, by the same discipline that excludes it for
types: arithmetic / relations between numbers (`K >= 2`, `N = 2*M`,
"power of two"). Those are rustc's job — the demo `HeapSort` carries a
`const { assert!(K >= 2) }` as the second validity layer. (See
`spec_core::tests::consts_default_explicit_enumerated_and_shared`.)

## Discipline (non-negotiable)

Registry constraints are **structural equality** (a shared variable: `a == b`)
and **role membership** (`x: Role`) ONLY. No arithmetic, conditionals, or
negation — that road is Prolog. If a constraint can't be expressed structurally,
it must be **pinned** explicitly, not queried.

## The compile stages

1. **Registry load** — `Registry::parse` turns the text catalog into a component
   graph (roles, slots, consts + value sets, projected params, imports).
2. **Query parse** — `parse_query` → a `Query` (bindings, refinements, holes).
3. **Solve** — `solve` lowers the query to a *set* of ground spec trees:
   environment cross-product, shared-variable threading, refinement role checks,
   quantifier sampling, canonical dedup, depth bound. **0 holes → 1, all holes →
   many, partial → a family — one code path.**
4. **Resolve + Emit** — each ground tree → concrete Rust type + label + imports
   (`resolve`), then Rust source (`emit_one` / `generate_table`).
5. **rustc** — full type checking + monomorphization. The authoritative gate.

Modes 1 (`spec_macro`, one inline tree) and 2 (`generator` / `build.rs`, a query
→ many trees) share stages 4–5 exactly; only the front-end differs.

## Crates

| Crate | Role |
|---|---|
| `registry.spec` | the text catalog — name → type/label templates, `provides` roles, slots, consts (+ value sets), `project`ed params, `uses` imports. |
| `spec_core` | pure-std engine. Modules: `registry`, `spec` (the `<…>` tree parser **and** the `parse_query` constraint front-end), `resolve`, `emit`, `enumerate` (the naive flat baseline), **`solve`** (the typed constraint solver). No syn/quote, no `rand`, no clock — randomness is a seeded SplitMix64. |
| `spec_macro` | mode 1 — inline `sort_spec!(Alias = …)`; a thin shell over `spec_core`. |
| `generator` | mode 2 — solve a query and print the Rust dispatch table. |
| `demo` | stub types mirroring real crate paths + both front-ends; `build.rs` solves several family queries into `generated::SORTS`. |

## What this closed, vs. the earlier flat prototype

The earlier prototype surfaced a real fork: flat `QuickSort<P, V, SS>` keeps the
pivot a **sibling** of the partition, so a per-slot role check can't express the
cross-slot arity constraint. `resolve` *accepted* `quick_sort<LL, ninther_dual>`
and only rustc rejected it; flat enumeration overproduced 42 quick variants
(~half arity-illegal), so the old `build.rs` had to **drop the whole quick_sort
family** before emitting.

The shared-variable query closes exactly that gap — it is the README's old
decision (c) ("a cross-slot constraint") expressed *structurally*, with no new
constraint vocabulary: just one variable used twice. The new `build.rs` emits
the **full 21-variant quick_sort family**, arity-correct, nothing dropped
(`demo::generated_table_runs`). The naive `enumerate` is kept as the contrast
baseline (`spec_core::tests::enumeration_overproduces_quicksort_arity_combos`
still asserts it *produces* the bad combo).

## Known prototype simplifications

- The registry is `include_str!`'d into the macro/generator; in production it's
  `Cargo.toml` metadata read in `build.rs`.
- The `spec_macro` inline front-end still uses the original `<…>` single-tree
  syntax and round-trips through `to_string()` (coarse error spans). The
  constraint language is the generator/build path; a production macro would keep
  `syn` spans.
- Random sampling **materializes** the depth-bounded population, then shuffles —
  exact and seeded, but it enumerates before it samples. For a genuinely large
  or unbounded grammar, production would rejection-sample derivations with a
  retry cap (same distinctness/clamp guarantees).
- Positional const **holes** are unsupported (a const hole must be a named
  argument so its value set is known); positional consts must be literals/vars.

## Planned next step (out of scope here)

- **rustdoc-JSON registry derivation** + a **`use`-probe registry validator**:
  derive the catalog (real type paths, generic arities, trait bounds) from the
  crate's rustdoc JSON, and validate every `uses` path / `provides` role against
  the actual crate by probe-compiling — so the registry can't drift from the
  code it claims to describe. The two-layer story stays: rustdoc/probe proves the
  catalog matches reality, the solver proves structure, rustc proves Rust.
- Input-language LSP tooling (Rust tooling on the *output* is enough for now),
  full backward unification beyond structural equality, and integrating against
  the real `array_vis_bench` crates (these stay faithful stubs).
