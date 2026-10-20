# Implementation plan: derive `uses` from the owning crate

Status: NOT STARTED. Designed 2026-10-20 from the concept review recorded in
`../todo.md`. Do it after `category_from_the_wrapper.md` and
`drop_slot_defaults_and_project.md`, since all three edit every fragment.

## Problem

Every component repeats its import path on a `uses` line: 201 paths across
the production fragments, 138 of the form `crate::Head` and 63 nested, such
as `heap_sort_lib::deep_heapify::Iterative`. The gather in
`spec_catalog/build.rs` already knows which crate declared each fragment, and
all but four of the nested heads are re-exported at their crate root.

Two things stop a plain `owning_crate::Head` rule.

- Baked foreign types in a template. `CombinedSelector<FirstElement,
  FirstElement>` in `heap.spec` needs three imports from three crates, and
  `HeapExtract<AryPair<Binary>, Iterative>` in `quick_heap.spec` names
  `AryPair`, which is not a component at all.
- Head collisions, where two different Rust types share a name: `Block`,
  `LeftLeftPartition` and `LeftRightPartition` exist in
  `heap_sort_lib::heap_partition` and in the partition leaf crates;
  `Classic` is a rod branching strategy and a shell gap sequence. The same
  collisions make a bare head in a query ambiguous today: `type_head_index`
  takes the first declared, and no query happens to name one of the four.

## Design

1. Heads are unique workspace wide. Rename the colliding Rust types:
   `HeapLeftLeft`, `HeapLeftRight` and `HeapBlock` in `heap_partition`, and
   `ClassicGaps` in shell. Labels do not change, so the catalog does not.
   `Registry` gains the invariant: two components with one type head must
   resolve to one path, else an error. This also closes the query ambiguity.
2. Every leaf re-exports its catalog types at the crate root. A glob
   re-export of `quick_deep_heapify` in `heap_sort_lib` closes the last four.
3. The registry records each component's owning crate. The build parses each
   fragment with its crate name (`Registry::parse_fragment(text, krate)`)
   and merges. `resolve` derives the imports: for every identifier in the
   type template that is not a `{slot}` hole, a literal or a keyword, the
   component whose head it is gives `its_crate::Ident`; the template's own
   head and any identifier no component declares resolve to the owning
   crate, which covers `AryPair`.
4. `uses` becomes optional, kept as an override, and is removed from the
   keyword table once no fragment needs it. The aim is zero.

## Steps

1. Tests first. In `Registry`: two components with one head and two paths
   fail. In `resolve`: a component with no `uses` resolves to `krate::Head`
   and a nested filler unions its crate.
2. Rename the four heads in Rust and in their fragments; add the uniqueness
   check. Golden unchanged.
3. Add the root re-exports, flatten the 63 nested paths to root form with a
   script. Golden unchanged.
4. Carry the owning crate through the gather; implement the derivation;
   strip `uses` from every fragment where the derived set matches, checking
   by diffing the emitted `use` lines before and after.
5. Keep `uses` only where derivation cannot reach, with the reason in a
   comment beside it; update the docs.

## Checks

- `catalog.golden` and the fingerprint unchanged at every step. The emitted
  `use` lines change from nested to root paths; rustc checks them.
- `grep -rhn '^\s*uses' crates --include=*.spec` is empty, or every hit has
  a comment.
