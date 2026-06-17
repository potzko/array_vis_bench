# Implementation plan: enumeration strategies (`finite` / `affine` / `sample` / `spread`)

Status: **DESIGNED, NOT STARTED.** Names + semantics locked 2026-06-17. Park here; pick up
when ready. This is a `spec_core` language extension, not a runtime feature.

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

(NB: `.` already exists as a per-hole "one" quantifier in `spec.rs`'s Quant parser — this lifts the
same idea to a positional rest-fill in the arg list.)

### Fill-set algebra (context; mostly already in the grammar)
Already in `avbs.pest`: `_` / `*` = maximal (`!{T}`), `|` union (today only inside `where(…)`),
`-` difference (`_ - {X}`), `{…}` set literals, `?N@seed` per-hole sample.
New gaps (Phase 4, low priority): bare `|` in value position, splat `*TG`, xor / intersection.
**Glyph collisions to resolve:** `*` already = exhaustive hole; `&` already = type-intersection
in `type_ann`.

## Phased implementation

Build order: **0 → 1 → 2** unblock the recursive quick-heaps with zero runtime machinery; then 3,
then 4. Each phase: grammar (if any) → lowering → solve → tests.

### Phase 0 — cycle analysis (foundational, the hard part)
- New `spec_core/src/cycles.rs`: build the role-dependency graph from the `Registry`
  (component → its slot roles → providers of each), run Tarjan SCC, expose
  `is_cyclic(component)` / `cyclic_providers(role)`.
- **Risk: medium.** The graph construction MUST mirror how `enumerate`/`solve` descends, or
  `finite` and the enumerator disagree. Unit-test on a hand-built recursive fragment
  (`A` has slot of role `R`; `B: R` has slot of role `A`'s role → cycle).

### Phase 1 — `affine` (self-terminating; no Phase 0 needed)
- `spec.rs`: add a strategy tag to `Query` (or a `QValue` wrapper).
- `avbs.rs`: recognize `affine(<value>)` at lowering, set the tag.
- `solve.rs` / `enumerate.rs`: thread a subtree-scoped `used: HashSet<canonical-component>`;
  prune already-used candidates; pop on backtrack.
- **Risk: low.** Terminates because each component is used ≤ once.

### Phase 2 — `finite`
- Uses Phase 0: at a `finite` family, the candidate set at each hole = providers-of-T MINUS
  `cyclic_providers(T)` (exactly `!{T} - !{T : cyclic}`).
- Keep the existing `depth N` directive as an optional backstop for deep-but-finite families.
- **Risk: low once Phase 0 lands.**

### Phase 3 — `sample` (optional seed) + `spread`
- `sample`: reuse `Take` distinct+dedup; seed optional → when `random`, draw from the global
  build RNG; when omitted, deterministic default seed.
- `spread`: stratified pick across the variant tree (round-robin over pre-order DFS).
- **Risk: low.** Wrinkle: `seed = random` ⇒ nondeterministic codegen for that family (count
  stable, identities vary; only fires on actual recompiles).

### Phase 4 — set-op polish (lowest priority, partly exists)
- Bare `|` union in value position; splat `*TG`; xor / intersection. Small `.pest` additions +
  glyph-collision resolution. Defer until 0–3 are solid.

## Open questions (defaults above unless changed)
1. `affine` scope: subtree (default) vs always-global?
2. Pairs: ordered (default) vs unordered?
3. `spread` stratification: round-robin over DFS (default) vs subtree-size-weighted?

## Pointers
- Memory: `project_enumeration_strategies.md`, `project_avbs_query_language.md`,
  `project_first_class_kinds.md`.
- Grammar: `spec_core/src/avbs.pest`; lowering `spec_core/src/avbs.rs`; solve
  `spec_core/src/solve.rs`; enumerate `spec_core/src/enumerate.rs`; types `spec_core/src/spec.rs`.
- Terminology sources: Substructural type systems (Walker, ATTAPL ch.1); Occurs check (Wikipedia);
  equirecursive/regular-tree types.
