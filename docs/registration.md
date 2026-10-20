# The catalog pipeline

How an algorithm announces itself without a central list. Every entry in
`array_vis_bench_core::bench_registry::ALGORITHMS` is emitted by
`spec_catalog`'s build script from two kinds of declarative file.

## The manifest handshake

A crate that contributes to the catalog says so in its `Cargo.toml`:

```toml
[package.metadata.array_vis_bench]
spec  = "pivot_first.spec"   # the catalog fragment this crate owns
query = "quick.avbs"         # optional: the program that builds its families
```

Paths are relative to the crate root. The crate must be in `spec_catalog`'s
dependency closure, which in practice means listing it in
`spec_catalog/Cargo.toml`. The build warns about a fragment it can see in the
workspace but not reach through dependencies.

## The `.spec` fragment

A fragment is a list of components. One component binds a Rust type to the
roles it provides and the slots it needs.

```text
component pk_single
  type     Pivoted<{scheme}, {pivot}>
  label    {scheme}<{pivot}>
  provides PartitionKind SelectPartition
  uses     quick_sort_lib::Pivoted
  slot     scheme PartScheme
  slot     pivot  PartPivot
end
```

| keyword | meaning |
|---|---|
| `component <name>` to `end` | one component; the name is unique across the whole workspace |
| `type <expr>` | the Rust type with a `{hole}` per slot; leaves have no holes |
| `label <template>` | the human name, same templating; becomes `AlgorithmEntry.name` and the picker label |
| `provides <Role> ...` | the roles slots match against; a component may provide several |
| `uses <path> ...` | module paths spliced into the emitted `use` block |
| `slot <name> <Role>` | a type parameter filled by any component of that role; a query fills every slot |
| `const <name> [values <v>...]` | a const generic, matched by name or position; the value list is the finite set a quantified hole ranges over, and a const without one takes only a literal |
| `menu <a / b>` | picker sub path beneath the category root |
| `max_input <n>` | cap on input size, a contract or a time budget; becomes `max_input_size`, which the picker, the sort battery and the benchmark all honour |
| `adaptive <bool>` | per family literal, not inherited from the type |
| `nondeterministic <bool>` | the trace varies between runs; becomes `AlgorithmEntry.nondeterministic` |

Roles are strings, finer grained than Rust traits. A role names a population,
so one trait is often sliced into several roles that queries can enumerate:
`PartitionScheme` backs `PartScheme`, `PartitionKind`, `SelectPartition` and
`HeapExtractPart`. A Rust type is declared once, and its `provides` line lists
every role it fills: `FirstElement` is one component that provides `PartPivot`,
`HeapPiv` and `BeapPiv`. Two components share a type only when the spec data
differs, such as a label or the role of a slot. Slots bind by role, and rustc type
checks the emitted composition, so a role that admits a wrong type is caught
at compile time rather than at run time.

## The `.avbs` query

The grammar is in `spec_core/src/avbs.pest`. A program is optional directives
followed by bindings of the form `Name: Type = Value`.

```text
depth 12

QuickSorts: SortingAlgorithm = SortingAlgorithm<CanonicalQuickSort<
    part = _ - {
        Pivoted<_, LastElement>,
        DualPivoted<CombinedSelector<LastElement, _>>,
        DualPivoted<CombinedSelector<_, LastElement>>
    },
    small_sort = _
>>
```

### Type wrappers

A binding whose type is one of the emit wrappers is emittable. The wrapper
sets the category, and it is the only place a category is named. Kind is a
property of where a type sits, not of the type: `Pivoted` is a root under
`PartitionAlg<_>` and a filler in a quick sort's `part` slot. A component
declares nothing about its kind, and a `category` line in a fragment is a
parse error that names the wrapper.

| wrapper | category | role enumerated by `_` |
|---|---|---|
| `SortingAlgorithm<X>` | Sort | the driver's own slots |
| `Selection<X>` | QuickSelect | the driver's own slots |
| `RotationAlg<_>` | Rotation | `Rotation` |
| `SmallSortAlg<_>` | SmallSort | `SmallSortKind` |
| `PartitionAlg<_>` | Partition | `PartitionKind` |
| `MergeAlg<_>` | Merge | `MergeKind` |
| `ScratchspaceAlg<_>` | Scratchspace | `ScratchspaceKind` |
| `List<Wrapper>` | none | aggregates earlier bindings into one set |
| `Mains<Set>` | none | names the consumers to run over a set |

A binding whose type is a plain role (`he: HeapExtractPart = _`) is a helper.
It is not emitted; later bindings in the same file use it by name.

### Values

| form | meaning |
|---|---|
| `_` or `*` | exhaustive: every component providing the slot's role |
| `?`, `?N`, `?N@seed` | one, or N, random distinct fillers, optionally seeded |
| `name = value` | fill a named slot |
| `Head<a, b>` or `Head<x = a>` | a concrete application, positional or named |
| `A - {X, Y}` | set difference, matched on the resolved type string |
| `_ where (A | B | C)` | union of curated alternatives |
| `value as x` | capture; a later `x` must resolve to the same filler |
| `finite(_)` | exhaustive minus any tree in which a component nests inside itself |
| `[A, B]` | a list of bindings, for `List<...>` |
| `depth N` | directive: bound the nesting depth of recursive roles |
| `N of @seed` | directive: keep N whole results |

The solver works with two constraints only: structural equality through a
shared variable, and role membership. There is no arithmetic and no
negation. Numbers are opaque tokens drawn from a declared value set.

### Bounding recursion

A role whose fillers contain a slot of the same role would unfold forever.
`finite(_)` applies an occurs check and keeps every acyclic tree, so a cyclic
role enumerates to a finite set. `depth N` is the backstop for deep but
finite nesting. The cardinality pass refuses a query whose bound is unbounded
or exceeds the budget before any enumeration runs.

## What the emit does per entry

For each ground type the backend writes a module containing the `Ty` alias,
the `AlgorithmEntry` static and a constructor that registers the picker
variant with its facet axes. The entry's `run_with_input` is the category's
driver and its `run_correctness` is the category's battery:

| category | driver | battery |
|---|---|---|
| Sort | `run_sort_with_input` | `sort_battery` and `sort_stability_battery` |
| Rotation | `run_rotation_with_input` | `RotationSuite` |
| Partition | `run_partition_with_input` | `PartitionSuite` |
| Merge | `run_merge_with_input` | `MergeSuite` |
| SmallSort | `run_small_sort_with_input` | `SmallSortSuite` |
| QuickSelect | `run_quick_select_with_input` | `SelectSuite` |
| Scratchspace | `run_scratchspace_with_input` | `ScratchspaceSuite` |

A sort's battery call carries the entry's `max_input` literal, so the battery
skips patterns longer than the cap.

The suites are impls of `array_vis_bench_core::suites::CorrectnessSuite`,
one per kind, parameterised over the kind's role trait.

## Adding a first class kind

A kind is a category with its own driver, battery and wrapper. The
scratchspace kind is the worked example on this branch and touched exactly
these places:

1. `Category` variant, `as_str`, inputs (`list_inputs`, `primary_input`) in
   `array_vis_bench_core/src/bench_registry.rs`.
2. `run_<kind>_with_input` and `<kind>_battery` in the same file.
3. A `<Kind>Suite` impl of `CorrectnessSuite` in `suites.rs`.
4. An entry body arm in `spec_emit/src/lib.rs`.
5. The wrapper name, its role and its category: one row of the `WRAPPERS`
   table in `spec_core/src/avbs.rs`.
6. A `Kinds: <Kind>Alg = <Kind>Alg<_>` binding in `spec_catalog/kinds.avbs`.

## Validation summary

| when | check | outcome |
|---|---|---|
| build | duplicate component name across fragments | error |
| build | fragment declared but outside the closure | warning |
| build | binding solves to zero rows | error |
| build | cardinality bound unbounded or over budget | error with the count |
| build | emitted type fails to type check | rustc error naming the module |
| test | per family and total golden counts | test failure |
| startup | duplicate name, duplicate picker path, primary input missing or doubled | panic before `main` |
