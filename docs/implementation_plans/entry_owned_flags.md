# Implementation plan: the entry owns its flags

Status: DONE (2026-10-20). Designed from the concept review recorded in
`../todo.md`, the smallest of the "entry is the single source of truth" plans.
Built as designed, with one choice the design left open: `sort_battery` takes
`Option<usize>`, so the emitter passes the same literal it writes into
`max_input_size`. The catalog golden did not move and the emitted entries and
their types are identical.

## Problem

Two facts about an entry lived in side lists keyed by its name.

- A list of `(name, cap)` pairs. `correctness::sort_battery` looked the cap up
  by name through a lookup function. The emitter wrote the component's
  `max_input` twice: into this list and into `AlgorithmEntry::max_input_size`.
- A list of names for the entries whose trace varies between runs. Its only
  reader was a catalog test, because the determinism property check it was
  written for had no reader any more.

The two hand registration macros, one per list, had no callers.

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
   `sort_battery` and no kind joins a slice other than `ALGORITHMS`; a
   nondeterministic component sets the field. In `spec_catalog`:
   `random_shell_opted_out_of_the_determinism_check` reads the field.
2. Add the field and fill it in all seven arms (false outside sorts).
3. Change the `sort_battery` signature to `(sort_fn, name, cap)`; update the
   emit and every other caller (`grep -rn 'sort_battery('` lists them).
4. Delete the slices, the lookups and the macros. Update the validation
   table in `docs/registration.md` and the cap paragraph in
   `docs/writing-components.md`.

## Checks

- `catalog.golden` unchanged.
- A tree wide search for the removed slices, lookups and macros finds nothing.
- The small sort battery still bounds its inputs at the threshold; the
  catalog test `each_caps_input_at_its_threshold` covers it.
