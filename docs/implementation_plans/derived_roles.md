# Implementation plan: derived roles

Status: NOT STARTED. Written 2026-10-28 as the eighth concept cleanup; it
replaces the "prelude of shared query bindings" idea in `../todo.md`.

## Problem

A role is a named population and a Rust type lists every population it joins
on its `provides` line. Where one population is another with a few members
removed, that relation is spelled by hand on every member:

```text
component pivot_first     provides PartPivot HeapPiv BeapPiv
component pivot_middle    provides PartPivot HeapPiv BeapPiv
component pivot_median3   provides PartPivot HeapPiv BeapPiv
component pivot_ninther   provides PartPivot HeapPiv
component pivot_last      provides PartPivot
```

The rule, "heap sorts try every pivot but last, beap sorts every pivot but
last and ninther", lives nowhere. Adding a pivot means remembering which
family tags to put on it, and the fourteen tag combinations in the specs
(`Pivot PivotSingle`, `HeapExtractPart SelectPartition`, `SmallSort
NonTrivialSmallSort FunSmallSort SmallSortKind`, `DeepHeapifyClassic
BeapDeepHeapifyClassic RecDeepHeapify` and so on) are each such a rule
written out member by member.

## Why not a query prelude

`../todo.md` proposed a prelude of bindings every query imports, so one
binding could name a population once. Slots bind by role inside the `.spec`
though: `heap_qdh_sequential` has `slot v HeapPiv`, and the query fills it
with `dh = _`. A prelude binding is not reachable from a slot, so every query
that fills the slot would have to restate the population at the hole, per
component, which is more text than the tags it would replace. The
population belongs where the slot can name it: in the registry.

## Design

A fragment may define a role from other roles and from component names:

```text
role HeapPiv         = PartPivot - pivot_last
role BeapPiv         = HeapPiv - pivot_ninther
role SelectPartition = HeapExtractPart | PartitionKind
```

- `role <Name> = <term> (('|' | '-') <term>)*`, evaluated left to right. A
  term is a role name or a component name. `|` adds the term's members, `-`
  removes them.
- A role is base or derived, never both. A base role appears on `provides`
  lines; a derived role is defined by exactly one `role` line in the whole
  workspace and never appears on a `provides` line. Either violation is a
  registry error that names the role.
- An unknown name in a definition and a definition that reaches itself are
  registry errors.
- `Registry::providing(role)` returns the same sorted component list for a
  derived role as it does for a base one, so the enumerator, the solver, the
  cardinality guard and the resolver need no change. Slots, queries and
  wrappers name derived roles exactly as they name base ones.
- A derived role lives in the fragment of the family that consumes it, next to
  the slot that names it: `HeapPiv` in `heap.spec`, `BeapPiv` in `beap.spec`.

## Steps

1. Tests first in `spec_core/src/registry.rs`: a `role` line parses; a derived
   role's `providing` equals the base population minus and plus its terms; a
   role both provided and derived, an unknown term, and a self reaching
   definition each fail with the role's name in the message; two fragments
   defining the same derived role fail at `from_fragments`.
2. Implement in `registry.rs`: parse `role`, store the definitions, resolve
   them in `providing` with a memo, validate at construction.
3. Convert the specs one tag family at a time, each its own commit, and after
   each run `cargo test --release -p spec_catalog`: the golden and
   `verify/catalog.types` must not move, because the populations are the same
   sets spelled differently. Start with the pivots, then the deep heapify
   tags, then the small sort tags, then the partition unions. Where a tag
   turns out not to be expressible as differences and unions of other
   populations, leave it as a base role and say so in the commit.
4. Docs: the keyword table and the roles paragraph in `../registration.md`,
   the role counts in `../todo.md`, and item 8 there marked done.

## Pointers

- Registry: `spec_core/src/registry.rs` (`Component.provides`, `providing`,
  `parse_fragment`, `from_fragments`).
- Tag survey: `grep -rh provides --include=*.spec crates spec_catalog`.
- Doc: `../registration.md`, the fragment table and the "Roles are strings"
  paragraph.
