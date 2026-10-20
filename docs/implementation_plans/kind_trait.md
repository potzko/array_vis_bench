# Implementation plan: one shape per kind

Status: DONE (2026-10-20). Designed from the concept review recorded in
`../todo.md`, the largest of the cleanups. Closes todo item 6. The catalog
golden did not move, and every field of every one of the 1580 emitted entries
is identical to the one before. Built as designed, with these choices the
design left open:

- `RunFnNoop` returns `Option<(usize, usize)>`. A partition's battery checks the
  boundary it returns, so the one shape carries it, and every other kind
  returns `None`.
- `PropHook` is `fn(Category, RunFnNoop, &str, Option<usize>)`. One slice serves
  every kind, so a hook is told the kind, which says what the index means, and
  the cap, which carries the threshold the small sort hook used to get.
- `sort_stability_battery` takes the cap as well. The emitter used to pass it
  none, so a capped stable sort was checked at 500 elements.
- Merge was the one kind besides sort with no suite, because `RotationMerge`
  lived in `merge_sort_lib`. The trait and `run_rotation_merge` moved to the
  traits crate, re-exported from `merge_sort_lib`, and `MergeSuite` joined.
- A small sort's threshold is its cap. All seven emitted small sorts declare
  `max_input` equal to their `THRESHOLD`, and the battery panics on a small
  sort that declares none.
- A `KINDS` row also carries the picker root and the complexity bounds a kind
  fixes instead of inheriting (a small sort's time, a scratchspace's all
  three). The row holds the bare category name, so `Category::` is spelled
  once in the template.
- `EmitBackend::wrappers` hands the table to `lower` and `lower_program` as a
  slice parameter. The `category` parse error in `registry.rs` still names
  wrappers as examples.

## Problem

A kind (a `Category`) is spread over seven places, listed in
`docs/registration.md`. In the code:

- `Category` with seven variants and `as_str`.
- Five input traits, five input entry structs of two shapes (`Vec<usize>`
  and `(Vec<usize>, usize)`), five linkme slices, and `list_inputs` and
  `primary_input` matches in which Sort, Partition and Scratchspace share
  `SORT_INPUTS`.
- Seven `run_<kind>_with_input` drivers, identical except for the input
  slice and whether an index is passed through.
- Seven `*FnPtr` types and seven batteries; six `*PropHook` types with six
  slices.
- Five `CorrectnessSuite` impls, with sorts excluded because they have no
  role trait.
- Seven emit arms in `spec_emit` of about sixty lines each, differing in the
  adapter signature, the driver name, the suite and the category literal.
- Seven wrapper names in `spec_core::avbs`, domain names hard coded in the
  domain agnostic crate.

`Category` appears at 25 sites in `bench_registry.rs` and 19 in `spec_emit`.

## Design

Every kind runs on an array plus one index. A sort ignores the index, a
rotation gets its split, a merge its midpoint, a quick select its target.
That is the one input shape and the one adapter shape:

```rust
pub struct InputEntry {
    pub name: &'static str,
    pub categories: &'static [Category],
    pub generate: fn(&RunConfig) -> (Vec<usize>, usize),
    pub primary: bool,
}
#[distributed_slice] pub static INPUTS: [InputEntry] = [..];

pub type RunFn = fn(&mut [usize], usize, &mut dyn SortLogger<usize>);
pub type RunFnNoop = fn(&mut [usize], usize, &mut NoOpLogger);
pub fn run_with_input(category: Category, input: &str, config: &RunConfig, f: RunFn, logger: &mut dyn SortLogger<usize>);
pub type PropHook = fn(RunFnNoop, &str);
```

`list_inputs` and `primary_input` become filters over the tags, and
`validate_registries` checks one primary per category from them. The small
sort threshold the battery needs is the entry's `max_input_size`.

The per kind pieces that stay per kind are the checks and the dispatch, and
they become data in one table in `spec_emit`:

```rust
struct KindAbi { wrapper: &str, category: &str, role: &str, dispatch: &str, suite: &str }
const KINDS: [KindAbi; 7] = [ ... ];
```

One `entry_body` template reads the row. `dispatch` is the adapter expression:
the inherent `sort` call for a sort, the `run_rotation` helper for a rotation.
`suite` is the `CorrectnessSuite` impl. The wrapper
list reaches `spec_core::avbs::lower` from the backend through
`EmitBackend::wrappers()`, so the compiler crate carries no domain names.

Sorts join `CorrectnessSuite` through a `DirectSort` role trait with a one
line impl per driver type, so every kind's `run_correct` is
`{Suite}::<Ty>::verify(NAME, cap)`.

A new kind then touches a `Category` variant, a row in `KINDS`, a
`<Kind>Suite` impl holding its battery, and its inputs' category tags. Four
one line or one impl places, down from seven files.

## Steps

1. Inputs. Test first: a parity test that `primary_input` and `list_inputs`
   return today's answers for all seven categories. Then one `InputEntry`,
   one slice, tags on every input in `inputs.rs`, the validator reading the
   tags.
2. Drivers. One `run_with_input`; the emit passes the category. Fingerprint
   unchanged.
3. Pointers and hooks. One `RunFnNoop`, one `PropHook` slice; batteries
   take `(f, name, cap)`; the small sort battery reads the cap.
4. Emit. The `KINDS` table and one template; the seven arms and the per kind
   `spec_emit` tests become table driven.
5. Wrappers from the backend; `kinds.avbs` is unchanged.
6. `DirectSort` and `SortSuite`; the sort arm's inline battery calls go.
7. Docs: the places to touch list in `docs/registration.md` and the kind
   section of `docs/writing-components.md`.

## Checks

- `catalog.golden` and the fingerprint unchanged at every step.
- A toy kind added in a `spec_emit` test touches the four places and emits.
- `grep -c 'Category::' spec_emit/src/lib.rs` counts the table rows and
  nothing else.
