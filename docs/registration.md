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
  slot     scheme PartScheme
  slot     pivot  PartPivot
end
```

| keyword | meaning |
|---|---|
| `component <name>` to `end` | one component; the name is unique across the whole workspace |
| `type <expr>` | the Rust type with a `{hole}` per slot; leaves have no holes |
| `label <template>` | the human name, same templating; becomes `AlgorithmEntry.name` and the picker label |
| `provides <Role> ...` | the base roles slots match against; a component may provide several |
| `role <Name> = <term> ...` | between components, a derived role: terms are role or component names, applied left to right, `\|` adds a term's members and `-` removes them; see [Derived roles](#derived-roles) |
| `uses <path> ...` | optional override of the derived imports, see [Imports](#imports) |
| `slot <name> <Role>` | a type parameter filled by any component of that role; a query fills every slot |
| `const <name> [values <v>...]` | a const generic, matched by name or position; the value list is the finite set a quantified hole ranges over, and a const without one takes only a literal |
| `menu <a / b>` | picker sub path beneath the category root |
| `max_input <n>` | cap on input size, a contract or a time budget; becomes `max_input_size`, which the picker, the sort battery and the benchmark all honour |
| `adaptive <bool>` | per family literal, not inherited from the type |
| `nondeterministic <bool>` | the trace varies between runs; becomes `AlgorithmEntry.nondeterministic` |

### Imports

A component lists no import paths. The emitted `use` block is derived from the
crate whose manifest points at the fragment. Every identifier in the `type`
template that is not a `{slot}` hole, a literal or a keyword is imported from
the crate root:

- an identifier that heads another component comes from that component's crate,
  so `CombinedSelector<FirstElement, FirstElement>` in `heap.spec` imports
  `CombinedSelector` from `quick_sort_lib` and `FirstElement` from `pivot_first`;
- the template's own head, and any identifier no component declares, such as
  `AryPair` in `quick_heap.spec`, come from the owning crate.

So every type a fragment names must be re-exported at the root of the crate
that declares it. A nested filler contributes its own crate's imports to the
entry, and a duplicate import is dropped.

Two components may share a type head only when the head is imported from one
path. Two different Rust types never share a head, which is why the heap
partitions are `HeapLeftLeft`, `HeapLeftRight` and `HeapBlock` and the shell gap
sequence is `ClassicGaps`. A head therefore names one type, in a query and in
an import.

`uses` is the override for a type the owning crate cannot reach. When a
component lists it, those paths replace the derived imports for that
component, and a comment beside the line gives the reason. `weak_heap_sort`
names `WeakHeapSort`, which lives in a crate that depends on the owner, and
`rdh_set` names `SequentialSet`, which lives in a sibling crate. A registry
parsed without an owning crate, as the unit test fixtures are, must list `uses`
on every component.

Roles are strings, finer grained than Rust traits. A role names a population,
so one trait is often sliced into several roles that queries can enumerate:
`PartitionScheme` backs `PartScheme`, `PartitionKind`, `SelectPartition` and
`HeapExtractPart`. A Rust type is declared once, and its `provides` line lists
every base role it fills: `FirstElement` is one component that provides
`PartPivot`, and it joins `HeapPiv` and `BeapPiv` through their `role` lines. Two
components share a type only when the spec data differs, such as a label or the
role of a slot. Slots bind by role, and rustc type checks the emitted
composition, so a role that admits a wrong type is caught at compile time
rather than at run time.

### Derived roles

A population that is another one with a few members removed or added is a
`role` line between components, not a tag on every member:

```text
role HeapPiv         = PartPivot - piv_last
role BeapPiv         = HeapPiv - piv_ninther
role SelectPartition = HeapExtractPart | PartitionKind
```

A term is a role name or a component name. Terms apply left to right, `|` adds
the term's members and `-` removes them, and a single term is an alias. A role
is base or derived, never both: a base role appears on `provides` lines, and a
derived role has exactly one `role` line in the whole workspace and never
appears on a `provides` line. The line lives in the fragment of the family that
consumes the role, next to the slot that names it, and may name the roles and
components of any fragment, because the registry resolves every definition once
all fragments are merged.

`Registry::providing` answers for a derived role exactly as for a base one, so a
slot, a hole, a refinement and a binding type take either. A new pivot that
provides `PartPivot` joins `HeapPiv` and `BeapPiv` on its own, and a `-` term in
that role's line keeps it out. A derived role is a list in one place, so the
kind roles that every crate tags to join the catalog (`SmallSortKind`,
`PartitionKind`, `MergeKind`, `Rotation`, `ScratchspaceKind`) stay base roles.

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

The wrapper table is the `KINDS` table of `spec_emit`, one row per kind. The
build hands it to `spec_core::avbs::lower` through `EmitBackend::wrappers`, so
the compiler crate names no kind.

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
| `Head<x = a, .>` | a bare `.` as the last argument fills every param left unspecified with one pick each |
| `A - {X, Y}` | set difference, matched on the resolved type string |
| `_ where (A | B | C)` | union of curated alternatives |
| `A \| B` | union of two families in value position, the same as `_ where (A \| B)`; `-` binds tighter, and a `where` alternative needs parentheses to hold one |
| `A /\ B` | intersection: the members both families hold, matched on the resolved type |
| `value as x` | capture; a later `x` must resolve to the same filler |
| `finite(_)` | exhaustive minus any tree in which a component nests inside itself |
| `affine(_)` | exhaustive, each component used at most once in the subtree under the wrapper |
| `sample(V, n = N)` | N random distinct variants of `V`; `seed = K` fixes the draw, `seed = random` redraws each build |
| `spread(V, n = N)` | N variants evenly spaced through the list of `V`, with no randomness |
| `[A, B]` | a list of bindings, for `List<...>` |
| `depth N` | directive: bound the nesting depth of recursive roles |
| `N of @seed` | directive: keep N whole results |

A bare `.` is the rest fill. `MergeSort<small_sort = _, .>` enumerates the small
sort and picks one filler for every other slot and one declared value for every
other const, so `MergeSort<.>` is one merge sort, the same one on every build,
because each pick is a `?` hole with the default seed. The `.` must be the last
argument and appear once, and it cannot fill a const that declares no `values`.
A bare positional `_` is not a rest fill: it is one exhaustive hole for the next
free param, so `QuickSort<_, _>` fills two. A param left out is an error that
names the slot, which means nothing fills the rest exhaustively; write `_` for
each param or `name = _` for each slot.

Families combine with three set operators, loosest first: `|` union, `/\`
intersection and `-` difference, so `A | B /\ C - D` is `A | (B /\ (C - D))`.
An operand is any value in type position, and the members are compared on the
resolved type, so `(A | B) /\ (B | C)` is `B`. An intersection keeps the first
operand's order, and a capture inside an operand rides out with the members that
survive. Symmetric difference needs no glyph of its own: `(A | B) - (A /\ B)`.
The glyphs are chosen to collide with nothing the language already spends. `&`
is the role intersection of a type annotation (`x: Sort & HasTimeBounds = ...`),
so the value intersection is `/\`. `*` is the exhaustive hole, and no splat
operator is defined: neither the plan nor the grammar says what its operand is.

The solver works with two constraints only: structural equality through a
shared variable, and role membership. There is no arithmetic and no
negation. Numbers are opaque tokens drawn from a declared value set.

### Bounding recursion

A role whose fillers contain a slot of the same role would unfold forever.
`finite(_)` applies an occurs check and keeps every acyclic tree, so a cyclic
role enumerates to a finite set. `depth N` is the backstop for deep but
finite nesting. The cardinality pass refuses a query whose bound is unbounded
or exceeds the budget before any enumeration runs.

`affine(_)` bounds it another way. Each component name may appear once in the
subtree under the wrapper, siblings included, so a recursive role terminates
with no cycle analysis and a composition that repeats a component is dropped.
`Combined<first, middle>` and `Combined<middle, first>` both stay, since pairs
are ordered, and `Combined<first, first>` goes. A component pinned outside the
wrapper does not count as used. `finite` and `affine` wrap a bare hole `_`, with
its quantifier if it has one.

### Cutting a family down

`sample(V, n = N)` keeps N distinct variants of the family `V` stands for,
drawn with the same seeded shuffle as a `?N` hole. Without a `seed` the draw is
fixed, so a build is reproducible. `seed = K` picks another fixed draw.
`seed = random` draws from one RNG that is seeded from the clock once per
build, so the variants change from build to build while the count stays N. A
test over a family that uses it may pin the count and never the variants.

`spread(V, n = N)` keeps the variants at positions `i * len / n` of the
family's list, which is the pre order walk of its trees, so they are evenly
spaced and the same on every build.

`n` is required for both. `V` is any value in type position, so
`sample(finite(_), n = 8)` and `spread(_ - {X}, n = 4)` work. An `n` past the
size of the family keeps all of it and warns.

## What the emit does per entry

For each ground type the backend writes a module containing the `Ty` alias
and the `AlgorithmEntry` static, which carries the picker `menu`, the facet
`axes` and the `label_template`. One template in `spec_emit` writes it for every
kind, reading the kind's row of the `KINDS` table. Both entry fns are the same
for every kind:

- `run_with_input` runs the one adapter `run_dyn(arr, index, logger)` through
  `bench_registry::run_with_input(category, ...)`. The index is whatever the
  input supplies: a sort ignores it, a rotation takes its split, a merge its
  midpoint and a quick select its target.
- `run_correctness` is `<{Kind}Suite<Ty> as CorrectnessSuite>::verify(NAME, cap)`,
  with the entry's `max_input` as the cap. Every battery skips patterns longer
  than the cap, and for a small sort the cap is its threshold.

| category | the adapter calls | suite | role trait |
|---|---|---|---|
| Sort | the inherent `sort` | `SortSuite` | `DirectSort` |
| Rotation | `run_rotation` | `RotationSuite` | `Rotation` |
| Partition | `run_partition_bounds` | `PartitionSuite` | `PartitionScheme` |
| Merge | `run_rotation_merge` | `MergeSuite` | `RotationMerge` |
| SmallSort | `SmallSort::sort` | `SmallSortSuite` | `SmallSort` |
| QuickSelect | `QuickSelect::select` | `SelectSuite` | `QuickSelect` |
| Scratchspace | `ScratchspaceRun::run` | `ScratchspaceSuite` | `ScratchspaceRun` |

The suites are impls of `array_vis_bench_core::suites::CorrectnessSuite`,
one per kind, parameterised over the kind's role trait. A sort driver type
joins `SortSuite` with one `array_vis_bench_traits::direct_sort!` line that
forwards the `DirectSort` role to its inherent `sort`.

## Adding a first class kind

A kind is a category with its own wrapper, role and suite. It takes four
places:

1. A `Category` variant, with its `as_str`, in
   `array_vis_bench_core/src/bench_registry.rs`.
2. A row of the `KINDS` table in `spec_emit/src/lib.rs`: the query wrapper and
   the role `_` enumerates under it, the category, the picker root, the call
   the adapter makes, the suite, and the complexity bounds the kind fixes
   instead of inheriting, if any.
3. A `<Kind>Suite` impl of `CorrectnessSuite` in
   `array_vis_bench_core/src/suites.rs`, which drives the kind's battery (a
   `<kind>_battery` beside the others in `bench_registry::correctness`).
4. The input tags: every shape in `array_vis_bench_core/src/inputs.rs` that
   serves the kind lists it in `categories`, and exactly one of them is
   primary for it. The startup validation panics otherwise.

Then a `Kinds: <Kind>Alg = <Kind>Alg<_>` binding in `spec_catalog/kinds.avbs`
asks for the entries. A `spec_emit` test adds a toy row to a local table and
emits a body for it, so the row is the whole of the emitter's side.

## Validation summary

| when | check | outcome |
|---|---|---|
| build | duplicate component name across fragments | error naming both crates |
| build | two components share a type head but import different paths | error naming both |
| build | component with neither an owning crate nor `uses` | error naming the component |
| build | a role on a `provides` line and also defined by a `role` line | error naming the role and a provider |
| build | a derived role defined twice, an unknown term, a term that is both a role and a component, or a definition that reaches itself | error naming the role |
| build | fragment declared but outside the closure | warning |
| build | binding solves to zero rows | error |
| build | cardinality bound unbounded or over budget | error with the count |
| build | emitted type fails to type check | rustc error naming the module |
| test | per family and total golden counts | test failure |
| startup | duplicate name, duplicate picker path, primary input missing or doubled | panic before `main` |
