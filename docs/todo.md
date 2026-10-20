# Status and roadmap

The one place that says what is built and what is open. Updated 2026-10-15
at the tip of `try_add_new_compiler_arch`. Design notes that used to live in
assistant memory files are no longer on this machine; the decision record
below is reconstructed from the docs, the plans and the commit history, and
this file is now the record.

## Where the compiler migration landed

The plan was to replace the metadata cross product generator with a typed
constraint compiler and reframe the repo as compiler plus ABI, with leaf
crates as the standard library, queries as the program, and each binary as a
link root. That is done:

- `spec_core` solves queries over a merged registry with shared variables,
  refinements, set difference, unions, captures, sampling, the cardinality
  guard and the occurs check (`finite`).
- `spec_emit` emits real `AlgorithmEntry` bodies for all seven kinds: Sort,
  QuickSelect, Rotation, Partition, Merge, SmallSort, Scratchspace.
- `spec_catalog` gathers every fragment and query through cargo metadata and
  is the sole source of `ALGORITHMS`. The combo generator, its wiring crate,
  the proc macro, the three standalone registries and the prototype are
  deleted.
- Every family moved: 1580 entries, each family's count pinned by a golden
  test. Quick, quick select and cyclent share one partition role system.
  Partitions, merges, rotations, small sorts and scratchspace workflows are
  first class kinds with their own batteries.
- The consumer surface exists in both languages: `Main` in Rust and
  `List<Mains<Set>>` in a query, run by `--consumers`.
- `cargo build --release` and `cargo test --release --workspace` are green
  as of this update.

What the plan called the `Visualisable` trait never appeared as a trait. The
category drivers plus the per category input slices fill that role, and the
`CorrectnessSuite` trait is its correctness half.

## Open work, in rough priority

1. Enumeration strategies. `finite` shipped; `affine`, `sample`, `spread`,
   the bare `.` rest fill and the set operator polish are not started.
   See [implementation_plans/enumeration_strategies.md](implementation_plans/enumeration_strategies.md).
2. Scratchspace, the remaining phases: the keys axis, a block merge sort
   family (WikiSort and GrailSort shapes), the buffer region logger event
   for the renderer, and block rolling so the bufferless backing is better
   than quadratic. See [implementation_plans/scratchspace_block_merge.md](implementation_plans/scratchspace_block_merge.md).
3. Sentinel quicksort. Parked until `compare_sorts` shows the canonical
   driver trailing the hand rolled Hoare baseline past the 1.05x gate.
   See [implementation_plans/sentinel_quicksort.md](implementation_plans/sentinel_quicksort.md).
4. The set quick select heap family shipped at minimal proof scope (two
   variants). Widening the inner select's partition role or the arities is
   a curation decision, not new machinery.
5. Benchmarking. The Criterion bench and its archive are gone; the
   `benchmark` consumer (`--timing [N] [K]` and `--consumers`) is the live
   benchmark and `compare_sorts` the gate. A persisted timing archive would
   give `plot_bench.py` a producer again and restore the N scaling chart;
   `MainReport::timings` holds the medians such an archive would write.
6. Unify sorts under `CorrectnessSuite`. Full sorts bypass the suite
   because they have no role trait, only an inherent `sort`. A `DirectSort`
   role trait would close the asymmetry noted in
   `array_vis_bench_core/src/suites.rs`.
7. `spec_emit` only supports types with an inherent `sort`. A type that
   implements only the legacy `SortAlgo` trait cannot be emitted. Either
   port the remaining users or add a catalog facet that switches the
   adapters.
8. Legacy residue to delete once nothing depends on it:
   - `crates/components/comb_ratio_components` and
     `crates/components/heap_internals_components`, metadata only crates
     for the retired generator with no consumer left;
   - the `SortAlgo` trait, once item 7 is settled;
   - `spec_catalog/quick_port.spec`, kept only as the baseline for the
     slotted versus baked equivalence test;
   - the `combo_codegen` directory still named in `.claude/settings.json`.
9. Publishability split. Do not start until there is a second genuinely
   independent consumer, the ABI (`sort_logger` plus
   `array_vis_bench_traits`) is frozen and versioned, and the leaves have
   stopped churning. The split is at the `Resolved` seam (type expression, label, uses): a
   domain agnostic solver front end, and the array domain backend that
   stays in the monorepo. Keep implementations, program and a
   consumer in one link unit so the inlining story holds.

## Concept cleanups, 2026-10-20 review

Seven more places where one fact is stated twice, each with a plan. The
recommended order is 1, 2, 3, then 4, 5, 6, then 7; the first three are an
afternoon each and remove concepts rather than move them.

1. The entry owns its flags. Done: `AlgorithmEntry` carries `nondeterministic`,
   `sort_battery` takes the cap as a parameter, and the two name keyed lists
   are gone.
   See [implementation_plans/entry_owned_flags.md](implementation_plans/entry_owned_flags.md).
2. Drop slot defaults and `project` from the spec grammar; no production
   query used either. Done.
   See [implementation_plans/drop_slot_defaults_and_project.md](implementation_plans/drop_slot_defaults_and_project.md).
3. Category from the wrapper. Done: the query wrapper is the one source of an
   entry's kind and a `category` line in a fragment is a parse error.
   See [implementation_plans/category_from_the_wrapper.md](implementation_plans/category_from_the_wrapper.md).
4. The entry owns its navigation. Done: `AlgorithmEntry` carries `menu`,
   `axes` and `label_template`, the picker view is built from `ALGORITHMS`,
   and the second registry crate and the four self registration features
   are gone.
   See [implementation_plans/entry_owned_navigation.md](implementation_plans/entry_owned_navigation.md).
5. One timing consumer. Done: `benchmark` is the one consumer that times,
   a fastest first table per category with `n` and `k` defaulting to 1000
   and 3; `--timing` overrides them and the speed test binary is gone.
   See [implementation_plans/one_timing_consumer.md](implementation_plans/one_timing_consumer.md).
6. Derive `uses` from the owning crate, after renaming the four colliding
   type heads. Done: heads are unique, and three components keep a listed
   `uses` where the owning crate cannot reach the type.
   See [implementation_plans/derive_uses_from_the_owning_crate.md](implementation_plans/derive_uses_from_the_owning_crate.md).
7. One shape per kind. One input entry, one run driver, one emit template
   over a kinds table, sorts under `CorrectnessSuite`.
   See [implementation_plans/kind_trait.md](implementation_plans/kind_trait.md).

## Role collapse, what held and what did not

A review of the specs found 205 components for 163 Rust types. The
duplicates were one type declared once per consumer role, for example a
pivot declared three times. Merging them leaves 169 components for 161
types, with all 1580 entries, names and concrete types unchanged. The
per entry identity is now pinned in `spec_catalog/catalog.golden`.

Two parts of the original proposal were checked against the code and
dropped:

- Roles cannot become plain Rust trait bounds. They are named
  populations: heap sorts try five pivots, beap sorts four and quick sort
  six, all implementing one trait. Expressing that in queries needs the
  same subtraction in every consuming query, and queries share no
  bindings across files. A prelude of shared bindings is the missing
  piece, and it should come first if the role tags are ever to go.
- Attributes on the Rust types cannot replace the spec. Slot roles,
  labels, menus and role tags are not derivable from generics and
  bounds, so the attribute would restate the spec line for line.

Three pairs of components still share a type on purpose. The beap and
heap quick deep heapify strategies differ in the role of their pivot
slot, and the beap ninther dual selector has its own label.

## Decision record

- The catalog is derived from per crate fragments, never a hand maintained
  production spec file. A crate owns its fragment and its query and points
  at them from its manifest.
- Complexity is inherited from the type through the composable traits. The
  catalog carries only what the type cannot: adaptive, max input, menu,
  nondeterministic. The category comes from the query wrapper. No Big-O
  literal and no arithmetic in a spec.
- The solver knows two constraints: structural equality through a shared
  variable, and role membership. Arity coupling is baked into composites
  (`Pivoted` versus `DualPivoted`) so an illegal pairing is unrepresentable,
  and rustc is a backstop, never the first line of defence.
- Recursion is bounded at solve time, declaratively. Runtime loop detection,
  per component recursion budgets and visit counters were rejected.
- Type heads are unique workspace wide, so a head names one Rust type in a
  query and in an import. A component's imports derive from the crate that
  owns its fragment, and a `uses` line is the override for a type that crate
  cannot reach.
- One registration path. Two emitters for the same name is a startup panic.
- Curate in the query. When one composition misbehaves, subtract it; do not
  patch the driver.
- A Rust type is one component. Its roles are the populations it joins.
- Standard and deferred quicksort are two families differing only in driver
  and small sort role.
- Enumeration strategy names are `finite` and `affine`; no seed means a
  deterministic build, and `seed = random` is an explicit opt in whose tests
  may pin counts but not identities.
- Per entry caps and the nondeterminism flag survive emission, as
  `max_input_size` and `nondeterministic` on the entry. A global test size
  over the whole registry runs exotic sorts out of memory.
- Inputs stay separate from algorithms. The compiler selects algorithms,
  the per category input slices supply inputs, and exactly one input per
  category is primary.
- Parallel work on families happens in separate worktrees with disjoint
  file slices. The shared points (the catalog manifest, the workspace
  members, the kinds query) are edited by the integrator only, and every
  merge is gated by the full test run.
