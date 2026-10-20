# Implementation plan: the entry owns its flags

Status: NOT STARTED. Designed 2026-10-20 from the concept review recorded in
`../todo.md`. Smallest of the "entry is the single source of truth" plans and
the one to do first.

## Problem

Two facts about an entry live in side lists keyed by its name.

- `SORT_TEST_CAPS` holds `(name, cap)` pairs. `correctness::sort_battery`
  looks the cap up by name through `max_n_for_tests`. The emitter writes the
  component's `max_input` twice: into this list and into
  `AlgorithmEntry::max_input_size`.
- `NONDETERMINISTIC_ALGOS` holds names. `is_nondeterministic` reads it, and
  its only caller in the tree is a catalog test. The determinism property
  check it was written for has no reader any more.

The two hand registration macros, `register_test_cap!` and
`register_nondeterministic!`, have no callers.

## Design

`AlgorithmEntry` gains `nondeterministic: bool`. `max_input_size` stays as
the one cap. `sort_battery` takes the cap as a parameter instead of looking it
up, and the emitter passes the literal it already has. Both slices, both
lookup functions and both macros go.

On the two meanings of the cap. 24 components declare `max_input`. For the
seven small sorts it is a contract: the type refuses longer input. For beap,
the chaos sorts and the set quick heap sort it is a time budget. Every
consumer treats the two the same way: the picker caps its prompt, the
battery skips longer patterns, the benchmark clamps, timing skips. Because the
behaviour coincides, one field is enough. The field's comment records both
meanings. Split it only when a consumer needs to tell them apart, for example
a timing run that wants slow sorts included at a reduced size.

## Steps

1. Tests first. In `spec_emit`: the sort body passes its cap to
   `sort_battery` and contains no `SORT_TEST_CAPS`; a nondeterministic
   component sets the field and emits no `NONDETERMINISTIC_ALGOS`. In
   `spec_catalog`: `random_shell_opted_out_of_the_determinism_check` reads
   the field.
2. Add the field and fill it in all seven arms (false outside sorts).
3. Change the `sort_battery` signature to `(sort_fn, name, cap)`; update the
   emit and every other caller (`grep -rn 'sort_battery('` lists them).
4. Delete the slices, the lookups and the macros. Update the validation
   table in `docs/registration.md` and the cap paragraph in
   `docs/writing-components.md`.

## Checks

- `catalog.golden` unchanged.
- `grep -rn 'SORT_TEST_CAPS\|NONDETERMINISTIC_ALGOS'` finds nothing.
- The small sort battery still bounds its inputs at the threshold; the
  catalog test `each_caps_input_at_its_threshold` covers it.
