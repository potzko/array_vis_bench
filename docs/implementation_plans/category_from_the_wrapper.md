# Implementation plan: category from the wrapper

Status: DONE (2026-10-20). Designed from the concept review recorded in
`../todo.md`. The emitted catalog is byte identical to the one before the
change, and `catalog.golden` is untouched.

## Problem

Two places state an entry's kind. The query wrapper (`SortingAlgorithm<X>`,
`RotationAlg<_>`, and the rest) and the root component's `category` line.
`lower_binding` turns the wrapper into the role `_` enumerates and drops the
wrapper name. `emit_entries` reads `comp.category` from the registry, and
`spec_emit::entry_body` defaults a missing category to Sort. Nothing compares
the two. A `SortingAlgorithm<X>` over a component declared `category
Rotation` emits as a rotation, silently.

87 components carry a `category` line: 58 Sort, 11 Rotation, 7 SmallSort, 4
Scratchspace, 3 Merge, 2 Partition, 2 QuickSelect.

## Design

The wrapper is the single source. Kind is a property of where a type sits,
not of the type: `Pivoted` is a root under `PartitionAlg` and a filler in
quick sort's `part` slot, and `NaiveRotationMerge` is a root under `MergeAlg`
and a filler in the rotation merge sorts. The binding knows; the component
cannot.

- One table in the lowering, `(wrapper, role, category)`, replaces
  `wrapper_role`.
- `NamedAlgo` carries the category. The build pairs each ground `SpecNode`
  with its family's category, and `emit_entries` takes those pairs.
  `EntryCtx.category` becomes `&str`; the default to Sort in `spec_emit`
  goes.
- `category` becomes a parse error in `Registry::parse` that names the
  wrapper, so a stale fragment fails loudly. The 87 lines are deleted.

`menu`, `max_input`, `adaptive` and `nondeterministic` stay on the component.
`menu` is arguably a property of the root too; it moves to the binding only
if a type ever needs two placements.

## Steps

1. Tests first, in `spec_core`: lowering the rotations binding of
   `kinds.avbs` yields a family whose category is `Rotation`;
   `emit_entries` given a category emits it for a component that declares
   none. The catalog golden guards the rest.
2. Thread the category: the table, `NamedAlgo.category`, the pairs through
   `spec_catalog/build.rs`, the `emit_entries` signature, `EntryCtx`, and
   the `spec_emit` tests that build an `EntryCtx` with `Some(..)`.
3. Delete the 87 lines with a script over `crates/**/*.spec`, make the
   keyword an error, update the keyword tables in `docs/registration.md` and
   `docs/writing-components.md` and the fragment comments that mention it.

## Checks

- `catalog.golden` unchanged.
- `grep -rn '^\s*category' --include=*.spec` finds nothing.
- A mismatch is no longer expressible: the component says nothing about
  kind, so there is nothing to disagree with.

## Outcome

- The table is `WRAPPERS` in `spec_core/src/avbs.rs`, one row per wrapper with
  its name, role and category. `is_wrapper_name` reads it too.
- The tree held 73 `category` lines in the production fragments, not the 87 the
  design counted: 44 Sort, 11 Rotation, 7 SmallSort, 4 Scratchspace, 3 Merge, 2
  Partition and 2 QuickSelect. The fixtures lost 9 (`registry.spec`) and 14
  (`quick_port.spec`).
- `slotted_decomposition_reproduces_the_baked_types` only solves and resolves,
  it never emits, so it needed no category of its own once the lines were gone.
- The `avbs` unit tests in `spec_core` run only with `--features
  pest-frontend`, which a workspace test run turns on through `spec_catalog`.
