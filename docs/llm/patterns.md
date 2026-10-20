# Patterns and conventions

Follow these when writing or modifying code here. The house style for code,
comments and commit messages is defined once in
`/home/omer/Documents/dev/skillz/STYLE.md`; this page covers what is
specific to this project.

## Algorithm bodies

All data access goes through the logger.

```rust
logger.cond_swap_gt(arr, i, ii);          // right
logger.write_data(arr, i, value);         // right
if arr[i] > arr[ii] { arr.swap(i, ii); }  // wrong: the visualiser never sees it
```

Reading into a temporary is fine. Auxiliary arrays are created and freed
through the logger (`create_aux_arr_t`, `free_aux_arr_t`) so the renderer can
draw them. Pass the same slice object consistently; the logger identifies an
array by its base pointer.

The signature is always `<T: Ord + Copy, U: ?Sized + SortLogger<T>>`. Keep
`?Sized` or the dyn path stops compiling. Never add a method level generic to
`SortLogger`; that would break dyn compatibility.

Flags are const generic booleans. Each combination monomorphises to its own
code path and the compiler removes the dead branches.

Partitions report unsorted ranges to a visitor; they never return them.

## Composition

- One idea per leaf. A leaf is a unit struct implementing one role trait
  plus the four annotation impls. Composers (`Pivoted<Scheme, Sel>`,
  `CombinedSelector<A, B>`) thread their parts' consts through
  `Complexity::sum` and `Complexity::product`.
- Write a monolith only when the algorithm is genuinely different; otherwise
  compose.
- Roles are finer than traits. Choose the roles a component provides by
  which queries should see it, not by which trait it implements.
- Curate in the query. A degenerate combination is removed with
  `_ - {...}` or `where (...)` in the `.avbs` file. Do not special case a
  driver for one bad combination.
- A role that can contain itself needs `finite(_)` or `depth N` in every
  query that enumerates it.

## Catalog hygiene

- Declare each Rust type once. List every role it fills on one `provides`
  line, and add a `category` if it is also a first class entry.
- Component names are a global namespace. Prefix them by what they are:
  `piv_` pivot, `pks_` partition scheme, `pk_` partition composite, `rot_`
  rotation, `mrm_` rotation merge, `ss_` small sort, `dss_` deferred small
  sort.
- Labels are lowercase, with the slot values in angle brackets:
  `quick sort<part: left-left<first>, small: size: 2>`.
- Never edit a list of algorithm names. Registration is derived.
- When a change moves a golden count in `spec_catalog/src/lib.rs`, print the
  new variant names, check they are the intended ones, then update the count
  and its comment.
- Keep the bench hot path monomorphic. No `dyn SortLogger` inside a loop.

## Testing

- Write the failing test first. A leaf carries one unit test that it does
  its job. A family's correctness comes from the emitted battery; a cap for a
  slow sort is declared, not patched around.
- `cargo test --release -p spec_catalog` is the fast check that the catalog
  still solves and every emitted entry still passes. `cargo test --release`
  covers the compiler and every crate.

## Git

- Work on a branch off the work branch, as small commits that each leave the
  tree green, then fold back. Commit messages are one or two lines saying
  the functional change, with no attribution line.
