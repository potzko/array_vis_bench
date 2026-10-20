# Implementation plan: drop slot defaults and project

Status: NOT STARTED. Designed 2026-10-20 from the concept review recorded in
`../todo.md`.

## Problem

Three features of the `.spec` grammar have no production user.

- Slot defaults. 62 of the 84 `slot` lines name a default filler. `resolve`
  uses it only when a query omits the slot, and every production query fills
  every slot with `_`, a binding or a literal.
- Const defaults. `const ping_pong false values false true` carries a default
  and a value set. Queries pass `_`, which ranges over `values`; the default
  is never read.
- `project`. A structural only parameter from before the partition
  composites (`project pivot PivotSingle` in `spec_core/registry.spec`).
  Arity is baked into `Pivoted` and `DualPivoted` now and no production
  component declares `project`. It survives as `ParamKind::Project` with arms
  in `solve.rs`, `cardinality.rs` and `resolve.rs`.

Each is a concept the grammar reader must learn and the solver must honour.

## Design

Remove all three. A slot line is `slot <name> <Role>`. A const line is `const
<name> values <v>...`, or `const <name>` for a literal only const. A query
that omits a slot is an error naming the slot and its role, which `resolve`
already raises for a slot without a default. `ParamKind` becomes `Type {
role }` and `Const { values }`.

The compiler's fixture `registry.spec` and its unit tests are the only
users. `const_defaults_apply` exercises `insertion<>`. The refinement tests
on `Partition[pivot = p]` in `avbs.rs` exercise refinements, not projection,
so the fixture gets a composite partition shaped like production and the
tests keep their meaning. `flat_quicksort_accepts_arity_mismatch_at_registry_level`
documents a gap the composites closed and goes with the feature.

## Steps

1. Tests first in `Registry::parse`: a `slot` line with a third word is an
   error, a `const` line with a bare default is an error, `project` is an
   unknown keyword.
2. Rewrite `registry.spec` to the production shape and adjust the unit tests
   that read it in `resolve.rs`, `lib.rs`, `cardinality.rs`, `avbs.rs` and
   `solve.rs`. Keep the fixture's population so the count assertions (two
   single pivots, four pivots in total, and so on) hold.
3. Strip the defaults from the production slot and const lines with a
   script; confirm the golden is unchanged.
4. Remove `ParamKind::Project`, the default fields and their arms; update
   the keyword tables in `docs/registration.md` and
   `docs/writing-components.md`.

## Checks

- `catalog.golden` unchanged.
- `grep -n Project spec_core/src` finds nothing.
- `spec_core` tests pass with the rewritten fixture.
