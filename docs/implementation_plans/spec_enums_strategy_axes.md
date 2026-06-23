# Implementation plan: enum / strategy axes (collapse deferred-vs-inplace, add language enums)

Status: **DESIGNED / IDEA, NOT STARTED.** Raised 2026-06-18. Motivated by the quicksort
driver proliferation: separate concern from [pivot_under_partition.md](pivot_under_partition.md)
(which collapses the pivoted/pivotless axis) — this one collapses the **inplace/deferred** axis.
Together they aim at one quicksort driver instead of the current four.

## The itch

Deferred vs inplace is a *binary behavioural toggle*, but today it's modelled as an entire
parallel sort family:

```
QuickSort<P, V, SS: SmallSort>            DeferredQuickSort<P, V, DSS: DeferredSmallSort>
PivotlessQuickSort<P, SS>                 DeferredPivotlessQuickSort<P, DSS>   ← step 1 just added these
```

Four driver types, two near-identical recursions, and **two disjoint small-sort role sets**.
The user's instinct: deferred-vs-inplace should be a lightweight marker (an empty type / an enum
variant), and the driver should just `match` on it — not a whole family.

```
enum quick_sort_strategy { inplace, deferred }      // user's sketch
```

## DECISION (2026-06-18): compile-time, presented as a drill-down tree

Keep it in the **compile-time** system — enum variants are **marker types** (flavour A below),
every leaf is one monomorphised concrete type, no runtime `match`. The value the user wants is not
a runtime toggle but a **nicer navigation**: the picker should drill down

```
quick sort
├─ pivoted                       partition = Pivoted<Scheme, Sel>
│  ├─ in-place                   Strategy = Inplace
│  │  └─ scheme → pivot → small sort                  ← "specifics"
│  └─ deferred                   Strategy = Deferred
│     └─ scheme → pivot → deferred small sort
└─ non-pivoted                   partition = HeapExtract<…> (self-splitting)
   ├─ in-place
   │  └─ heap kind / arity → small sort
   └─ deferred
      └─ heap kind / arity → deferred small sort
```

backed by one type `QuickSort<Partition, SmallSort, Strategy>` (all type params; `Strategy` a unit
marker). Key point: **menu order is the picker's choice, independent of how the type nests.** The
picker walks the axes pivotedness → deferredness → specifics even though, in the type, pivotedness
lives inside `Partition` and `Strategy` is the last param. This is the structural-nav machinery
([noble-mixing-patterson](../../.claude/plans/noble-mixing-patterson.md)) plus a chosen axis order.

The two early levels are **different axis shapes**, both navigable:
- **pivoted / non-pivoted = a structural fork.** The branches have *different* specifics
  (scheme+pivot vs heap arity). In the type it's "which kind of `Partition`"; surfaced as a
  partition sub-family (a `pivoted` / `pivotless` classification tag — itself an enum, so the
  construct does double duty).
- **deferred / in-place = an orthogonal enum axis** (a marker type) present on both branches. It
  also *filters* the specifics: after `deferred`, only deferred small sorts are offered (the
  disjoint-set coupling below stops being awkward — the drill-down makes it feel natural).

So flavour B (runtime value-enum) is **dropped**; the rest of this doc keeps it only as the
rejected alternative for context.

## What "enum" means in this language (two distinct flavours)

The catalog already has the *primitive* of a closed scalar axis: a `Const` param with a finite
`values` set ([spec_core/src/registry.rs](../../spec_core/src/registry.rs) `ParamKind::Const`) —
e.g. a `ping_pong = true|false` const-generic hole fans over exactly `{true,false}`. A named enum
is the symbolic generalisation of that (named variants instead of ints/bools). There are two ways
to realise it, and they are genuinely different — the choice is the crux of this plan:

### Flavour A — **type-enum (marker types)**: a closed role of unit types
Each variant is a unit struct implementing a marker trait; the enum is a **closed role**
(single definition site; build error if another crate `provides` it; variants are leaves with no
slots). It fans into a **type slot** exactly like any role and monomorphises.

```rust
pub trait SmallSortStrategy { /* hooks, see below */ }
pub struct Inplace;  impl SmallSortStrategy for Inplace  { … }
pub struct Deferred; impl SmallSortStrategy for Deferred { … }
```

- Pros: zero new machinery in solve/emit — it's "a role whose providers happen to be unit types,"
  reusing the whole cross-product path. Zero runtime cost (monomorphised). Fits the
  existing "everything is a monomorphised type composition" architecture.
- Cons: still **2× monomorphisations** of the driver body (one per strategy). Doesn't actually
  shrink generated code — it shrinks *source* (one driver type instead of two), which is most of
  what the user wants, but not the binary.

### Flavour B — **value-enum (runtime)**: the first NON-type axis
Variants are values of a real Rust `enum`; the driver takes one as a runtime arg and `match`es
(the user's "in the actual sort we just match over this"). The emit bakes the chosen variant into
each entry's call, so the catalog still has one entry per variant — but the heavy generic body is
monomorphised **once** and both entries call it with a different enum value.

```rust
enum Strategy { Inplace, Deferred }
fn quick_sort_recursive<…>(arr, …, strat: Strategy) {
    if small { match strat { Inplace => SS::sort(arr, l), Deferred => {} } ; return; }
    … partition + recurse(…, strat) …
}
fn sort<…>(arr, l, strat: Strategy) {
    recurse(arr, …, strat);
    if let Strategy::Deferred = strat { SS::final_pass(arr, l); }
}
// two entries: |a,l| sort(a,l,Inplace)   and   |a,l| sort(a,l,Deferred)
```

- Pros: **1× monomorphised driver body** for the inplace/deferred axis (≈2× code-size saving on
  this driver), one recursion to maintain, and it's literally the `match` the user pictured. The
  branch is perfectly predicted / hoisted out of the hot path by the optimiser.
- Cons: it's a genuinely new axis *kind* — the language today only has type slots and const
  literals; a runtime-enum axis is the first "scalar config baked into the entry call." Touches
  emit (entry body passes the value), the picker (a choice that isn't a type), and naming.

**Decided: flavour A (type-enum / marker types)** — see the DECISION section above. The driver
*type* duplication still goes away (one `QuickSort<Partition, SmallSort, Strategy>` source), even
though the body still monomorphises per strategy; the binary-size saving of B was judged not worth
introducing the language's first non-type axis. Flavour B retained above only as rejected context.

## The honest caveat: strategy is NOT cleanly orthogonal to the small sort (today)

The two small-sort surfaces are different operations, not the same op invoked differently
([array_vis_bench_traits/src/role/small_sort.rs](../../array_vis_bench_traits/src/role/small_sort.rs)):

- `SmallSort::sort(slice) -> bool` — sort one small sub-slice **during** recursion.
- `DeferredSmallSort::final_pass(whole)` — one **windowed** sweep over the *entire* array at the
  end (`windowed_insertion_sort_with`, O(N·THRESHOLD)) — not a per-slice sort.

And the component SETS are disjoint:
- inplace: `{size:1, size:2, insertion:32, network:16}`
- deferred: `{deferred-insertion linear/binary} × {16, 32}`

So you cannot just cross `strategy × small_sort` and keep 448 — `size:1` has no sensible deferred
form, deferred-binary has no inplace form. To get the *full* benefit the enum promises, the small
sort itself wants remodelling into `(algorithm × threshold)` with a **unified trait exposing both
`sort(slice)` and `final_pass(whole)`**, and a `match`-on-strategy choosing which to call. But not
every algorithm supports both (a sorting network has no windowed final pass), so a "which
strategies does this small sort support" coupling reappears — the same shape as the pivot↔partition
arity coupling, enforced in the spec (a deferred-incapable small sort simply isn't offered under
`strategy = deferred`).

**Implication:** the enum/strategy axis is clean to *add*; making it pay off fully is gated on the
small-sort remodel. Two-stage it (below) so stage 1 lands value independently.

## Phasing (compile-time / marker-type track)

1. **Unify the driver behind a `Strategy` marker type (Rust only).** Add unit markers
   `Inplace` / `Deferred` + a `QuickSortStrategy` trait with the two hooks
   (`on_small(arr)` during recursion, `finalize(arr)` at the end); `Inplace::on_small = SS::sort`,
   `finalize` = no-op; `Deferred::on_small` = skip, `finalize` = `DSS::final_pass`. One generic
   driver `QuickSort<Partition, SmallSort, Strategy>`; `DeferredQuickSort` /
   `DeferredPivotlessQuickSort` deleted as separate *types*. The strategy co-selects which
   small-sort trait is in play for now (no remodel yet). Behind existing tests.
2. **Language `enum` construct (closed, type-enum).** Parser: `enum Name { variant … }` desugars to
   a closed role of unit-type components (build error if another crate `provides` it). It fans into
   a type slot via the existing cross-product. A `pivoted` / `pivotless` classification on the
   Partition role is the second enum (drives the structural fork). No new axis kind — reuses roles.
3. **Drill-down navigation (the actual ask).** Picker presents enum axes as early levels in a
   chosen order (pivotedness → deferredness → specifics), grouping by the classification tags. This
   is the structural-nav work; enum axes supply the early grouping levels, recursive nav supplies
   the specifics. Naming renders the path, not a flat slot list.
4. **Small-sort remodel (unlocks full orthogonality, optional).** Collapse `SmallSort` +
   `DeferredSmallSort` into one trait with `sort(slice)` + optional `final_pass(whole)` + a
   `SUPPORTS_DEFERRED` flag; spec offers `strategy = deferred` only for supporting small sorts
   (arity-style coupling). Then `strategy × small_sort` is a real (coupled) product instead of two
   hand-curated disjoint sets — though the drill-down menu already makes the disjoint sets read
   cleanly, so this is a nice-to-have, not a blocker.

## Interaction with the other plans
- [pivot_under_partition.md](pivot_under_partition.md): that collapses pivoted/pivotless into the
  partition; this collapses inplace/deferred into a strategy. Do both ⇒ ONE driver
  `QuickSort<Partition, SmallSort, Strategy>` (or `<Partition, SmallSort>` + runtime strategy).
- [enumeration_strategies.md](enumeration_strategies.md): `finite`/`affine`/`sample`/`spread`
  operate over whatever axes exist; a value-enum axis is just another finite set they can sample.
- Naming/picker for a non-type axis overlaps the structural-nav work
  ([noble-mixing-patterson](../../.claude/plans/noble-mixing-patterson.md)).

## Pointers
- Drivers: [crates/families/quick_sort_lib/src/quick_sort.rs](../../crates/families/quick_sort_lib/src/quick_sort.rs),
  [deferred_quick_sort.rs](../../crates/families/quick_sort_lib/src/deferred_quick_sort.rs).
- Small-sort traits: [array_vis_bench_traits/src/role/small_sort.rs](../../array_vis_bench_traits/src/role/small_sort.rs).
- Spec parser / `Const` finite-values precedent: [spec_core/src/registry.rs](../../spec_core/src/registry.rs).
- Quick spec/avbs (the two driver components + disjoint small-sort sets):
  [crates/families/quick_sort_lib/quick.spec](../../crates/families/quick_sort_lib/quick.spec).
</content>
</invoke>
