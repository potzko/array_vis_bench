# Implementation plan: enumeration strategies (`finite` / `affine` / `sample` / `spread`)

Status: BUILT, with two items left out. Names and semantics locked 2026-06-17. `finite`
shipped 2026-07-01 (commit e6edd80, `enumerate_finite` in `spec_core/src/enumerate.rs`) and is
load bearing in `quick_heap.avbs`. `affine`, `sample`, `spread`, the `.` and `..` rest fills
and the set operators `|` and `/\` shipped 2026-10-28. Left out: the splat `*TG` and a glyph
for xor, each for a reason given below. This is a `spec_core` language extension, not a
runtime feature.

## Problem

A query enumerates algorithm variants as typed compositions (a generic head with role-typed
holes; `_` = "all implementors of this role"). Two failure modes of naive exhaustive enumeration:

1. **Recursion → infinite.** Some fillers contain a hole of their own role, so the type
   expansion loops forever (the real case: `heap_extract_partition` → `heap` → `deep_heapify`
   → `quick_select_deep_heapify` → `partition` → `heap_extract_partition` → …). This is why the
   hard quick-heap families were deferred.
2. **Degenerate redundancy.** e.g. `dual_pivot<pivot_l = first, pivot_r = first>` — a pointless
   repeat.

Resolution: bound the enumeration **declaratively at solve time** via set-filtering strategies.
A constrained-exhaustive enumeration yields a FINITE set of FINITE types, which then run fine —
**no runtime guard / `max_visits` / depth-magic needed.** (We rejected per-component recursion
budgets and runtime loop detection in favor of this.)

## The constructs (all solve-time)

Builtins wrapping a family value (like `Mains`/`List`), reusing the `application` paren grammar,
so MOST need no `.pest` change — just lowering recognition + a strategy on the `Query`.

- `finite( <family> )` — exhaustive MINUS types whose expansion graph is cyclic.
  `= !{T} - !{T : cyclic}`. **The actual fix for the recursion cycle.** Established term: the
  **occurs check** (rejects a type containing itself) over **recursive/regular-tree types**;
  detection = SCC / back-edge cycle detection over the role→component→slot→role graph, run
  BEFORE enumerating. Keeps bounded use of `heap_extract_partition`, cuts only the infinite
  self-nesting.
- `affine( <family> )` — exhaustive, each component used at most once in a composition.
  Established term: **affine** (substructural types: linear = exactly once, affine = at most
  once, relevant = ≥ once, ordered). Self-terminating; no graph analysis. (Was the user's old
  "attempt B"; their "attempt A" was a path-scoped variant of this.)
- `sample( <family>, n = N, seed = K | random )` — N random distinct variants. Reuses the
  existing `Take` ("N of @seed", distinct + deduped) logic, lifted to a value wrapper.
- `spread( <family>, n = N )` — N evenly-split variants (stratified; round-robin over the
  pre-order DFS of the variant tree).

### Locked decisions
- **Names:** `affine` and `finite` (chosen over `acyclic` / `distinctQ`).
- **Seed model:** no seed = DETERMINISTIC default (reproducible builds, Cargo cache intact).
  `seed = random` = explicit opt-in pulling from ONE global compile-time RNG, seeded once per
  build. A family using `seed = random` must NOT have a test pinning exact variant identities
  (count N stays stable; identities vary per build).
- **`affine` scope (default):** subtree rooted at the wrapper (composable; at the root ⇒ whole
  build). Revisit if a global-only form is wanted.
- **Pairs (default):** ORDERED — `Combined<first, middle>` ≠ `Combined<middle, first>`.

### Wildcard rest-fill sugar (decided 2026-06-17)
A bare wildcard in a slot list means "fill every *unspecified* hole":
- bare `_` → fill the rest **exhaustively**. `quick_sort<_>` = all quicksorts;
  `quick_sort<partition = LL<pivot = _>, _>` = pin the partition family, fill the rest exhaustively.
- bare `.` → fill the rest with **one concrete pick each** (a singleton). `merge_sort<.>` = one
  (random) merge sort.

Built 2026-10-28, with the exhaustive fill under its own glyph. As the last argument `.`
expands at lowering into a `?` hole with the default seed for every param the arguments leave
unspecified, a declared value for a const and a filler for a slot, so `merge_sort<.>` is one
merge sort and the same one every build; `..` expands the same way into an exhaustive hole, so
`merge_sort<..>` is every merge sort. A bare `_` stays what it was, one exhaustive hole for the
next free param, so `quick_sort<_>` fills `partition` and nothing else, and a param left out
entirely is an error naming the slot. The exhaustive fill could not take `_` as the plan said,
because `_` was taken, and `..` reads as "the rest" the way Rust spells it.

(NB: the legacy `spec.rs` parser reads `.` as the EXHAUSTIVE hole and `?` as the one pick, the
opposite of what this section first said. The AVBS grammar only reads a `.` as an argument of
its own, so the two never meet.)

### Fill-set algebra (context; mostly already in the grammar)
Already in `avbs.pest`: `_` / `*` = maximal (`!{T}`), `|` union (today only inside `where(…)`),
`-` difference (`_ - {X}`), `{…}` set literals, `?N@seed` per-hole sample.
New gaps (Phase 4, low priority): bare `|` in value position, splat `*TG`, xor / intersection.
**Glyph collisions to resolve:** `*` already = exhaustive hole; `&` already = type-intersection
in `type_ann`. Resolved 2026-10-28: the value intersection is `/\`, `&` stays the role
intersection of a type annotation, and `*` stays the hole, with no splat spending it.

## Phased implementation

Build order: **0 → 1 → 2** unblock the recursive quick-heaps with zero runtime machinery; then 3,
then 4. Each phase: grammar (if any) → lowering → solve → tests.

### Phase 0 — cycle analysis (foundational, the hard part)
Outcome: not built as a separate pass. `finite` runs the occurs check while enumerating (a
child whose component is already on the ancestor path is skipped), which yields the same
acyclic set without a graph module. Revisit only if `affine` needs shared infrastructure.
- New `spec_core/src/cycles.rs`: build the role-dependency graph from the `Registry`
  (component → its slot roles → providers of each), run Tarjan SCC, expose
  `is_cyclic(component)` / `cyclic_providers(role)`.
- **Risk: medium.** The graph construction MUST mirror how `enumerate`/`solve` descends, or
  `finite` and the enumerator disagree. Unit-test on a hand-built recursive fragment
  (`A` has slot of role `R`; `B: R` has slot of role `A`'s role → cycle).

### Phase 1, `affine` (self-terminating, no Phase 0 needed) DONE 2026-10-28
- `spec.rs`: add a strategy tag to `Query` (or a `QValue` wrapper).
- `avbs.rs`: recognize `affine(<value>)` at lowering, set the tag.
- `solve.rs` / `enumerate.rs`: thread a subtree-scoped `used: HashSet<canonical-component>`;
  prune already-used candidates; pop on backtrack.
- **Risk: low.** Terminates because each component is used ≤ once.
- Built as `QValue::Affine`, lowered like `Finite`, with `enumerate_affine` threading the
  used set through siblings as well as ancestors. It wraps a bare hole `_` only, like `finite`.

### Phase 2 — `finite` DONE 2026-07-01
- Uses Phase 0: at a `finite` family, the candidate set at each hole = providers-of-T MINUS
  `cyclic_providers(T)` (exactly `!{T} - !{T : cyclic}`).
- Keep the existing `depth N` directive as an optional backstop for deep-but-finite families.
- **Risk: low once Phase 0 lands.**

### Phase 3, `sample` (optional seed) + `spread` DONE 2026-10-28
- `sample`: reuse `Take` distinct+dedup; seed optional → when `random`, draw from the global
  build RNG; when omitted, deterministic default seed.
- `spread`: stratified pick across the variant tree (round-robin over pre-order DFS).
- **Risk: low.** Wrinkle: `seed = random` ⇒ nondeterministic codegen for that family (count
  stable, identities vary; only fires on actual recompiles).
- Built as `QValue::Sample` and `QValue::Spread`, lowered from the `n` and `seed` named
  arguments of the application, over any value in type position. `seed = random` draws from
  a `OnceLock` RNG in `solve.rs`, seeded once per process from the clock and the process id.
  `spread` takes positions `i * len / n` of the deduped variant list.

### Phase 4, set-op polish (lowest priority, partly exists) DONE 2026-10-28, in part
- Built: the bare `|` union (lowered to the `where` union over an exhaustive hole) and the
  intersection `/\` (`QValue::Inter`, matched on the canonical type, counted in the size guard).
  Precedence, loosest first: `|`, `/\`, `-`. A `where` alternative splits on `|`, so one that
  holds a union is parenthesised.
- Left out: the splat `*TG`, because `TG` is not defined by this plan or the grammar and the
  operator has no meaning to implement. Xor, because `(A | B) - (A /\ B)` already writes it.

## Open questions (defaults above unless changed)
1. `affine` scope: subtree (default) vs always-global?
2. Pairs: ordered (default) vs unordered?
3. `spread` stratification: round-robin over DFS (default) vs subtree-size-weighted?

## Pointers
- Status and decision record: [../todo.md](../todo.md).
- Grammar: `spec_core/src/avbs.pest`; lowering `spec_core/src/avbs.rs`; solve
  `spec_core/src/solve.rs`; enumerate `spec_core/src/enumerate.rs`; types `spec_core/src/spec.rs`.
- Terminology sources: Substructural type systems (Walker, ATTAPL ch.1); Occurs check (Wikipedia);
  equirecursive/regular-tree types.
