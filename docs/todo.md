# Migration plan: constraint compiler as core

> Living plan for the `try_add_new_compiler_arch` refactor. Replace the
> `combo_codegen` cross-product generator with the typed constraint compiler
> (`spec_core`, prototyped in `spec_compiler_proto/`), reframing the repo as
> **core = compiler + ABI**, with the stdlib (leaf crates), the program (a
> query), and the consumers (visualiser / bench / tests) as users of that core.
>
> Companion docs: [architecture.md](architecture.md), [registration.md](registration.md),
> [trait-system.md](trait-system.md), [adding-a-sort.md](adding-a-sort.md).
> Design rationale & decision record: memory `project-compiler-consumer-architecture`.
>
> **Parked follow-on plans:** [implementation_plans/](implementation_plans/) — designed-but-not-started
> work.
> - [enumeration_strategies.md](implementation_plans/enumeration_strategies.md) — `finite`/`affine`/`sample`/`spread`
>   to bound recursive variant enumeration (the quick-heap cycle) + the `_`/`.` rest-fill sugar.
> - [pivot_under_partition.md](implementation_plans/pivot_under_partition.md) — move pivot off the
>   base quicksort into a self-pivoting partition (heap-extract stops faking a pivot; arity coupling
>   becomes structural).

## North star

The catalog is the **language**, a query is the **program**, `spec_core` is the
**compiler**, the leaf crates are the **standard library** the program links
against, the ABI (`SortLogger` + role traits + `AlgorithmEntry`) is the
**calling convention** the compiler defines, and each consumer is a separate
`main` that links a (possibly different) compiled program.

**Working discipline:** `cargo test --workspace` green at *every* checkpoint.
One family at a time. `combo_codegen` and `spec_core` coexist during migration
(dedup emitted entries by name) until the last family moves.

---

## The gap that drives everything (read first)

The prototype's `emit` produces a toy dispatch table:

```rust
pub const SORTS: &[(&str, fn(&mut [usize]))] = &[ ("quick[...]", s0), ... ];
```

…and it type-checks only against `demo/`'s `PhantomData` stub
(`struct QuickSort<P,V,SS>; fn sort(arr: &mut [usize])`). The **real** runtime
contract is `AlgorithmEntry` ([bench_registry.rs:482](../array_vis_bench_core/src/bench_registry.rs#L482)),
emitted today by `sort_family!` ([sort_family.rs:805](../sort_registry_macro/src/sort_family.rs#L805)):

```rust
// per ground sort, inside its own module:
fn run_default(input_name: &str, config: &RunConfig, logger: &mut dyn SortLogger<usize>) {
    fn sort_dyn(arr: &mut [usize], logger: &mut dyn SortLogger<usize>) { <Ty>::sort(arr, logger); }
    run_sort_with_input(input_name, config, sort_dyn, logger);
}
fn run_correct() {
    correctness::sort_battery(<Ty>::sort_noop, NAME);
    correctness::sort_stability_battery(<Ty>::sort_noop, NAME, STABLE);
}
#[distributed_slice(ALGORITHMS)]
static ENTRY: AlgorithmEntry = AlgorithmEntry {
    name: NAME,
    category: Category::Sort,
    worst:   <Ty as HasTimeBounds>::WORST,   // inherited from the type, not the catalog
    best:    <Ty as HasTimeBounds>::BEST,
    average: <Ty as HasTimeBounds>::AVERAGE,
    space:   <Ty as HasSpace>::SPACE,
    stable:  <Ty as HasStability>::STABLE,
    adaptive: <literal — NOT compositional, per-family>,
    max_input_size: None,                    // Some(N) for small-sorts etc.
    run_with_input: run_default,
    run_correctness: run_correct,
};
#[ctor::ctor]
fn register() { register_sort_variant(NAME, &["sorts", <cat..>], &[(<facet_role>, <facet_val>)..]); }
```

So `emit` must learn to produce **that** block, the catalog must carry the
fields the type can't supply (`category`, `adaptive`, `max_input_size`, menu
facets), and the per-type properties (`worst/best/average/space/stable`) come
from the **composable traits** (`array_vis_bench_traits::{HasTimeBounds, HasSpace,
HasStability}`, [composable.rs](../array_vis_bench_traits/src/composable.rs)) via
`<Ty as Trait>::CONST`. De-hollowing this is Phase 0 and gates everything else.

`Category` ([bench_registry.rs:33](../array_vis_bench_core/src/bench_registry.rs#L33)):
`Sort · Rotation · Partition · Merge · SmallSort · QuickSelect` — each has its
own `run_with_input` driver + correctness battery + input slice.

---

## Phase 0 — De-hollow the center *(in `spec_compiler_proto/`, against a faithful ABI)*

> **STATUS (commit `ae1442a`):** 0.1–0.3 + 0.5 done; Sort driver (0.4) done and
> green (`avb_abi` + `emit_entries`, 30 `AlgorithmEntry` rows run through
> `&mut dyn SortLogger` + pass correctness). Partition/Merge/Rotation drivers
> (rest of 0.4) are the in-flight **fleet tasks** (`fleet/*-driver` worktrees).

Make "implementations + a program = one program" true against the **real** ABI
shape, before touching the main workspace. Reversible; the proto stays detached.

- [ ] **0.1 Faithful ABI stub crate** `spec_compiler_proto/avb_abi/`. Mirror —
  with identical field names & signatures so emitted code is copy-portable to
  the real crates later:
  - `Category` enum (6 variants), `Complexity` (mirror `complexity.rs`:
    `UNKNOWN`, `from_str`), `RunConfig { size, seed, .. }`.
  - `SortLogger<usize>` (a small but representative dyn-compatible subset:
    `create_arr`, `write_data`, `cmp`, `swap`); `NoOpLogger`; a capture logger
    that records a `Vec<Event>` (stands in for `VisualizerLogger`).
  - `SortAlgo<T, U: SortLogger<T>> { fn sort(arr, logger) }` ([sort_traits.rs:14](../array_vis_bench_traits/src/sort_traits.rs#L14)),
    `HasTimeBounds`/`HasSpace`/`HasStability` with the same defaulted consts.
  - `AlgorithmEntry` (all 11 fields), `#[distributed_slice] static ALGORITHMS`
    (add `linkme` + `ctor` as proto deps — fidelity over purity here),
    `run_sort_with_input`, `register_sort_variant`.
  - Update `demo/`'s stub sort types to `impl SortAlgo + HasTimeBounds + …`.
- [ ] **0.2 Extend the catalog schema** (`spec_core::registry`) — additive keywords:
  - `category <Sort|Partition|…>` per family/root component.
  - `adaptive <true|false>` (per-family literal; not derived).
  - `max_input <N>` → `max_input_size: Some(N)` (e.g. small-sorts).
  - `facet <role> <template>` → the `register_sort_variant` menu pairs.
  - complexity stays **inherited** (`<Ty as HasTimeBounds>::WORST`); no Big-O in
    the catalog (keep the discipline — no arithmetic, no value reasoning).
- [ ] **0.3 New emit backend** `spec_core::emit::emit_entry` (keep `emit_one`/
  `generate_table` for the demo's old table + tests). Per ground sort: emit the
  module block above. `worst/…/stable` as `<Ty as Trait>::CONST`; `category`,
  `adaptive`, `max_input_size`, facets from the catalog.
- [ ] **0.4 Category-dispatched drivers.** `category` selects the
  `run_with_input` template + correctness battery (Sort→`run_sort_with_input`+
  `sort_battery`; Partition→partition driver, cf.
  [quick_partition_registry/src/lib.rs:103](../crates/registries/quick_partition_registry/src/lib.rs#L103);
  etc.). One template per category.
- [ ] **0.5 Tests** (`spec_core` + `avb_abi`):
  - emitted entries compile & link; `ALGORITHMS` is populated (count == solved
    sorts); names unique.
  - `(entry.run_with_input)("...", &cfg, &mut capture)` produces a non-empty
    event log → the emitted program actually drives the dyn logger.
  - arity-unrepresentable guarantee still holds (port
    `shared_pivot_makes_arity_mismatch_unrepresentable`).

**DoD:** `cd spec_compiler_proto && cargo test` green, no warnings; a generated
`AlgorithmEntry` runs a sort via `&mut dyn SortLogger` and emits a real log.

## Phase 1 — The `Visualisable` contract

Pin the consumer-side dual of the ABI, generalised to sub-components.

- [ ] **1.1** Define `Visualisable` (object-safe), and document that
  `AlgorithmEntry` *is* its type-erased registry record and the `*_INPUTS`
  slices ([bench_registry.rs:168](../array_vis_bench_core/src/bench_registry.rs#L168))
  are its input recipes, keyed by `category`:
  ```rust
  trait Visualisable {            // "given my inputs, drive the logger"
      type Input;
      fn demo_inputs() -> &'static [Self::Input];
      fn run(input: &Self::Input, logger: &mut dyn SortLogger<usize>);
  }
  ```
- [ ] **1.2** Emit `Category::Partition` and `Category::Rotation` entries from
  queries (a partition is Visualisable because it drives the logger) — proves
  visualisation composes below the whole-sort level.

**DoD:** a partition query → an `AlgorithmEntry{category: Partition}` the (stub)
visualiser drives via `PARTITION`/`ROTATION` inputs.

## Phase 2 — Promote `spec_core` into the main workspace *(no behaviour change)*

- [ ] **2.1** Move `spec_core` (+ `spec_macro`, + `avb_abi` folded into the real
  crates — it was only a stub) into the root workspace `members` +
  `default-members`. `spec_core` sits where `combo_codegen` sits: a
  `[build-dependencies]` of `array_vis_bench_full`. `combo_codegen` stays for now.
- [ ] **2.2** Keep `spec_compiler_proto/` as a detached conformance harness (it
  proved the solver) — or delete `demo/` once the real path subsumes it. Decide,
  don't drift.

**DoD:** `cargo build --workspace` green; `spec_core`'s unit tests run under the
root `cargo test`; binary/bench/visualiser byte-identical.

## Phase 3 — Catalog from per-leaf metadata *(one source of truth)*

Avoid the registry-drift trap: do **not** hand-maintain a `.spec` file in
production. Derive the catalog from the metadata the leaves already declare.

- [ ] **3.1** Teach `spec_core` to build its `Registry` from
  `[[package.metadata.array_vis_bench.components]]` blocks (e.g.
  [partition_lomuto/Cargo.toml](../crates/partitions/partition_lomuto/Cargo.toml))
  via the existing `combo_codegen::scan_workspace_components` walk
  ([array_vis_bench_full/build.rs](../array_vis_bench_full/build.rs)). Reuse it
  or lift it into a shared `catalog_source` module.
- [ ] **3.2** Extend the metadata schema **additively** so `combo_codegen` keeps
  working: add `slots`, projected params (the arity markers — `project pivot
  PivotSingle/Dual`), `category`, `adaptive`, `max_input`, `facet`. Verified
  fact: published crates keep `[package.metadata]` and `cargo_metadata` reads it,
  so this path survives a future publish split (no change needed then).
- [ ] **3.3** registry.spec text format remains for `spec_core`'s own unit tests
  only.

**DoD:** `spec_core::solve` over the derived catalog reproduces one family's
variant set (pick `shell` or `merge`) — same names, same ALGORITHMS membership —
vs. `combo_codegen`, modulo ordering.

## Phase 4 — Pilot: move one arity-free family off `combo_codegen`

- [ ] **4.1** Pick a family with no cross-slot arity constraint (`shell_sort` or
  `merge`). In `array_vis_bench_full/build.rs`, emit it via
  `spec_core::solve + emit_entry`; exclude it from `combo_codegen`'s output.
- [ ] **4.2** Assert parity: a test that the emitted `AlgorithmEntry` names for
  that family == the previous set; all correctness batteries pass; the menu tree
  + bench rows are unchanged.

**DoD:** `cargo test --workspace` green; pilot family identical end-to-end.

## Phase 5 — Quick-sort family + subsume the standalone registries *(the payoff)*

The arity-correct quick cross-product via the shared-pivot query, replacing
`combo_codegen`'s quick output **and** the hand-written registries.

- [ ] **5.1** Generate quick via the shared-`p` query (LL+dual never built).
- [ ] **5.2** ⚠️ **Single registration path, same commit.** The three standalone
  registries register straight into `ALGORITHMS` via `distributed_slice` + `ctor`
  and are force-linked by `#[used]` anchors in `array_vis_bench_full/src/lib.rs`.
  Turning on `spec_core` emission for the same variants *while they still link* =
  duplicate-name `AlgorithmEntry` → startup panic. In one commit: emit via
  `spec_core` **and** drop the corresponding registry's contributions + its
  `#[used]` anchor. Targets:
  - `quick_partition_registry` (`register_partition!` × ~30, `Category::Partition`,
    [lib.rs:170+](../crates/registries/quick_partition_registry/src/lib.rs#L170))
  - `quick_select_registry` (`Category::QuickSelect`)
  - `merge_standalone_registry` (`Category::Merge`)
- [ ] **5.3** Add a test: `ALGORITHMS` has **no duplicate names** (guards the
  single-path invariant permanently).

**DoD:** quick / partition / quick-select / merge menus + bench + correctness
unchanged; no duplicate entries; LL+dual absent by construction.

## Phase 6 — Retire `combo_codegen`

- [ ] **6.1** Move all remaining families to `spec_core`. Remove the `family!()`
  text-scan markers and `combo_codegen::scan`/`scan_workspace_*` calls.
- [ ] **6.2** Delete `combo_codegen` + the three registry crates; drop from
  workspace `members`/`default-members`. Keep `sort_family!`/`sort_registry_macro`
  only if still used for hand-written one-offs; otherwise retire too.

**DoD:** `combo_codegen` gone; `cargo test --workspace` green; binary, bench,
and visualiser behaviour identical to pre-migration.

## Phase 7 — *(later, gated)* Publishability / workspace split

Do **not** start until the conditions in the architecture memory hold (a second
genuinely-independent consumer, frozen+semver'd ABI, leaves stopped churning).

- [ ] **7.1** Split `spec_core` at the `Resolved { type_expr, label, uses }` seam:
  generic solver front-end (publishable, domain-agnostic) vs `avb_emit` (the
  sort/`AlgorithmEntry` backend — stays in the monorepo).
- [ ] **7.2** Freeze + semver the ABI (`sort_logger` + `array_vis_bench_traits`).
- [ ] **7.3** Compiler → own workspace / published crate. Stdlib leaves
  publishable when stable. Dev iteration via `[patch.crates-io]` → local paths
  (keeps everything source-local; LTO + metadata discovery + linkme all
  preserved — verified). Keep impls + program + a consumer in one final link unit
  so `codegen-units=1` + ThinLTO still cover the hot path (the `<1.05×` heap gate).

---

## Cross-cutting invariants (hold in every phase)

- **One registration path.** Never let two emitters register the same name into
  `ALGORITHMS`. The dedup-by-name test (5.3) is the guard.
- **Inherit complexity; declare the rest.** `worst/best/average/space/stable`
  come from `<Ty as Has*>::CONST` (composable traits); `category`, `adaptive`,
  `max_input_size`, and menu facets come from the catalog. No Big-O literals or
  arithmetic in the registry (the structural-only discipline).
- **Respect per-entry test caps.** `SORT_TEST_CAPS` / `max_n_for_tests`
  ([bench_registry.rs:577](../array_vis_bench_core/src/bench_registry.rs#L577))
  and `NONDETERMINISTIC_ALGOS` opt-outs must survive emission — a global N over
  `ALGORITHMS` OOMs exotic sorts (see memory `feedback-correctness-test-memory`).
- **Inputs stay separate.** `SORT_INPUTS`/`ROTATION_INPUTS`/`MERGE_INPUTS`/
  `SMALL_SORT_INPUTS`/`QUICK_SELECT_INPUTS` are the `Visualisable` input recipes;
  the compiler selects algorithms, not inputs.
- **Deterministic emit order** so generated output diffs are stable.
- **One workspace, one binary** through Phase 6 → `codegen-units=1` + ThinLTO
  keep inlining the monomorphic hot path; don't fragment the link unit.

## Parallelization & worktree strategy (multiple agents at once)

The macro-phases are a **sequential spine**; the parallelism is in the wide middle.

- **Trunk — single-threaded, one hand: Phases 0 → 2 → 3.** Never fan out a shared
  foundation; an incoherent `emit_entry`/ABI contract is worse than a slow one.
- **Fan-out — parallel: Phases 4–5, one agent per family** (~15: quick, merge,
  shell, heap, insertion, comb, bubble, cycle, beap, weak_heap, rod, circle,
  fun_sorts, quick_select, quick_heap), each in its **own git worktree**. Also
  Phase 3.2's ~50 per-leaf metadata edits.
- **Close — single-threaded: Phase 6** (delete `combo_codegen` + registries).

**Hard rules so parallel-green ⇒ integrated-green:**
1. **Worktree isolation is mandatory.** Agents sharing one tree clobber each
   other (`Agent` tool `isolation: "worktree"`, or `git worktree` per branch).
2. **Disjoint file slices.** A family agent edits only that family's crate +
   query file. The shared contention points — `array_vis_bench_full/build.rs`,
   the `#[used]` anchors in `array_vis_bench_full/src/lib.rs`, root `Cargo.toml`
   `members`/`Cargo.lock`, and the single `ALGORITHMS` slice — are **serialized
   to the integrator**, never edited inside a parallel agent.
3. **Serial integration gate.** After merging each family branch:
   `cargo test --workspace` + the no-duplicate-`ALGORITHMS`-names test (5.3).

**Make the trunk fan-out-friendly (do this in Phase 3/4):** drive `build.rs`
from *data* — per-leaf metadata + per-family query files, iterating a family
list — so migrating a family adds/edits a file, never the shared `build.rs`.
`combo_codegen` already emits per-`source_module`; preserve that so each family's
generated output is its own file.

**Mechanisms, by fit:** `Agent` tool worktree subagents (orchestrated here, now);
the `Workflow` tool (discover families → transform-each-in-worktree → verify-each
→ synthesize; opt-in); `git worktree` + branch-per-family for human teammates.
Speedup is bounded by the serial integration tail (`cargo test --workspace`).

## Prototype status (already done, `spec_compiler_proto/`)

Solver with shared variables + refinements; quantifiers `*`/`?`/`?N@seed` +
`N of`; seeded sampling, canonical dedup, clamp+warn; depth-bounded recursion;
projected params making arity mismatches unrepresentable; const defaults / value
sets / shared const vars. 14 `spec_core` + 5 `demo` tests green, clippy-clean.
**Missing for production:** the real `AlgorithmEntry` emit (Phase 0).
