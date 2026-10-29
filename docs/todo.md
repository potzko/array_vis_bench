# Status and roadmap

The one place that says what is built and what is open. Updated 2026-10-28
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
- `cargo build --release` and `cargo t` (nextest, release) are green
  as of this update.

What the plan called the `Visualisable` trait never appeared as a trait. The
one run driver plus the tagged `INPUTS` slice fill that role, and the
`CorrectnessSuite` trait is its correctness half, one impl per kind.

## Open work, in rough priority

1. Enumeration strategies. `finite`, `affine`, `sample` and `spread` shipped:
   `finite` and `affine` wrap a bare hole, `sample` and `spread` wrap any
   value in type position, and `seed = random` is seeded once per build from
   the clock. The rest fills `.` (one pick for every unspecified param) and
   `..` (an exhaustive hole for each) and the set operators `|` and `/\`
   shipped too. Not built, by decision: the splat `*TG`, whose operand nothing
   defines, and a glyph for xor, which `(A | B) - (A /\ B)` already writes.
   See [implementation_plans/enumeration_strategies.md](implementation_plans/enumeration_strategies.md).
2. Done: the scratchspace phases. The `Keys` axis, the block merge sort
   family (the WikiSort and GrailSort shapes, the buffer in the array or
   outside it by six sizes, each under every rotation), the buffer region
   logger event with its shading in the renderer, and a bufferless
   merge by rotations, `O(n log n)` moves instead of quadratic. See
   [implementation_plans/scratchspace_block_merge.md](implementation_plans/scratchspace_block_merge.md).
3. Dropped: the sentinel quicksort. Its trigger was the canonical driver
   trailing the hand rolled Hoare baseline past the 1.05x gate, and every
   gate run reads the driver at or under the baseline.
4. The set quick select heap family shipped at minimal proof scope (two
   variants). Widening the inner select's partition role or the arities is
   a curation decision, not new machinery.
5. Done: the `benchmark` consumer (`--timing [N] [K] [--input NAME]` and
   `--consumers`) is the live benchmark and `compare_sorts` the gate. It
   warms each entry up, builds the input outside the clock, batches runs
   under fifty microseconds, and reports the median with the inter
   quartile range and the Tukey outlier count. `--timing --out PATH`
   writes the medians as a TSV archive under a header naming its setup,
   and both archive kinds end with a line per entry giving its category and
   axes. `verify.sh` compares one timing archive against its baseline, and
   `plot_bench.py` writes the benchmark page from any mix of them: facet
   checklists with live counts, time against size per input shape, best of
   a facet, dominance over a size range, the effect of a facet and parallel
   coordinates, the view kept in the URL. View files in the query language,
   which would set that view from a named set of entries and inputs, are
   the next step. `--sweep SECONDS`
   climbs every entry up the size ladder within a time budget and finds the
   largest size each fits, which is what makes a time budget `max_input`
   unnecessary for the benchmark; the caps that remain are contracts or
   serve the picker and the batteries. The budget pays for building the
   inputs too, a batch's inputs hold at most a million elements, and no
   climb passes `--max-size`, 4 Mi by default, so an entry whose run does
   not grow with the size stops there instead of exhausting memory.
6. Done: sorts join `CorrectnessSuite` through the `DirectSort` role, one
   `direct_sort!` line per driver type. See
   [implementation_plans/kind_trait.md](implementation_plans/kind_trait.md).
7. Done: every sort dispatches through its inherent `sort` and the
   `SortAlgo` trait is gone; the comb ratio was its last implementor.
8. Done: the empty metadata crates of the retired generator and the
   `combo_codegen` path in `.claude/settings.json` are gone.
   `spec_catalog/quick_port.spec` is not residue, it is the fixture of the
   slotted versus baked equivalence test and stays.
9. Publishability split. Parked by decision (2026-10-28): the workspace
   stays one tree until either a release is wanted, which forces the split,
   or Rust grows reflection convenient enough to replace the generated
   catalog. Until then do not start it, and when it starts the
   preconditions hold: a second genuinely independent consumer, the ABI
   (`sort_logger` plus `array_vis_bench_traits`) frozen and versioned, and
   the leaves no longer churning. The split is at the `Resolved` seam (type expression, label, uses): a
   domain agnostic solver front end, and the array domain backend that
   stays in the monorepo. Keep implementations, program and a
   consumer in one link unit so the inlining story holds.

10. Instruction counts for the code generation gate. Not started. Count the
    instructions of the three `compare_sorts` pairs and one entry per kind
    under callgrind, pin them exactly, and fail a pair past two percent, so
    the zero cost claim rests on a deterministic number beside the timing.
    See [implementation_plans/instruction_counts.md](implementation_plans/instruction_counts.md).
11. Done: rod sort axes. A tournament merge that makes every strategy
    N log N and the strategy's gap chain as a Shell sequence, after two speed
    fixes to the merges; the tuned strategy was dropped, `log-parity` beats
    every constant setting. Open: the hold and shift kernel for the shell
    family, which the rod gap entries need to beat the recursion. See
    [implementation_plans/rod_sort_axes.md](implementation_plans/rod_sort_axes.md).

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
7. One shape per kind. Done: one `InputEntry` slice tagged by category, one
   `run_with_input` driver, one `RunFnNoop` and one hook slice for the
   batteries, one emit template over the `KINDS` table, the wrapper table
   supplied by the backend, and sorts under `CorrectnessSuite` through the
   `DirectSort` role. A new kind is a `Category` variant, a `KINDS` row, a
   `<Kind>Suite` impl and the input tags.
   See [implementation_plans/kind_trait.md](implementation_plans/kind_trait.md).
8. Derived roles. Done: a `role` line in a fragment defines a population as
   other roles and components added and removed, where a slot can name it.
   Eight roles are derived (`HeapPiv`, `BeapPiv`, `BeapDeepHeapifyClassic`,
   `RecDeepHeapify`, `SmallSort`, `NonTrivialSmallSort`, `FunSmallSort`,
   `SelectPartition`), leaving 37 base roles of the 45 in the crate
   fragments. The kind roles stay base, since every crate tags one to join the
   catalog. This replaced the earlier idea of a query prelude, which a slot
   cannot reach.
   See [implementation_plans/derived_roles.md](implementation_plans/derived_roles.md).

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
  bindings across files. A derived role states the subtraction once in the
  registry, where every slot can name it.
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
- A Rust type is one component. Its roles are the populations it joins. A
  population that is other populations plus or minus a few members is a
  `role` line, not a tag on each member.
- Standard and deferred quicksort are two families differing only in driver
  and small sort role.
- Enumeration strategy names are `finite`, `affine`, `sample` and `spread`;
  no seed means a deterministic build, and `seed = random` is an explicit opt
  in whose tests may pin counts but not identities.
- Per entry caps and the nondeterminism flag survive emission, as
  `max_input_size` and `nondeterministic` on the entry. A global test size
  over the whole registry runs exotic sorts out of memory.
- Inputs stay separate from algorithms. The compiler selects algorithms,
  the one `INPUTS` slice supplies inputs tagged by category, and exactly one
  input per category is primary.
- Parallel work on families happens in separate worktrees with disjoint
  file slices. The shared points (the catalog manifest, the workspace
  members, the kinds query) are edited by the integrator only, and every
  merge is gated by the full test run.
