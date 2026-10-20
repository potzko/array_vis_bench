# Writing components for `array_vis_bench`

A guide for adding sorting-algorithm machinery to this project.

This is not a normal "add a function and call it" codebase. You write **small,
single-purpose trait impls** (a pivot selector, a partition, a small sort), declare
each one **once** in a `.spec` file, and a constraint compiler multiplies them into
the full cross-product of concrete algorithms , hundreds of catalog entries , with no
hand-written glue. Get the two halves right (the Rust impl *and* its `.spec`/`.avbs`
declaration) and your one type shows up in every quicksort, quick-select, and
quick-heap variant automatically.

> **The single most important thing to internalise:** a perfect trait impl that isn't
> declared in a `.spec` file produces **nothing**. The Rust code is half the job.

## Contents

1. [What the project is, and what the compiler buys you](#1-what-the-project-is)
2. [The mental model: traits vs roles vs the compiler](#2-mental-model)
3. [Anatomy of a component: Rust impl + `.spec` + `.avbs`](#3-anatomy-of-a-component)
4. [The logger contract and instrumentation discipline](#4-the-logger-contract)
5. [Common patterns](#5-common-patterns)
6. [The trait catalog](#6-the-trait-catalog)
7. [Getting it into the catalog: crates, discovery, build, tests](#7-getting-it-into-the-catalog)
8. [Worked example: add a pivot selector end to end](#8-worked-example)
9. [Advanced: recursion bounding and curate-in-query](#9-advanced)

---

<a name="1-what-the-project-is"></a>
## 1. What the project is, and what the compiler buys you

`array_vis_bench` visualises and benchmarks sorting algorithms. Sorts run on `[usize]`
arrays; every observable operation (compare, swap, write, allocate) is recorded through
a **logger**, and the recording is either thrown away (benchmark path) or rendered to an
MP4 (visualiser path).

The interesting part is the *catalog*. There are about 1600 concrete algorithms, but almost
none of them are written out by hand. A quicksort is a cross-product:

```
partition scheme  x  pivot selector  x  small-sort cutoff  x  {in-place | deferred}
```

Rather than write that product as code, you write each **factor** once , one
`LeftLeftPartition`, one `FirstElement`, one `Size2SmallSort` , and a **spec-constraint
compiler** enumerates the combinations at build time, type-checks each one, and emits a
`catalog_entries.rs` full of `AlgorithmEntry` values registered into a global
`ALGORITHMS` slice.

What that buys you:

- **Add a factor, get the whole row for free.** A new pivot selector instantly appears
  in all 408 quicksorts, all 56 quick-selects, etc. , no edits to any driver.
- **Illegal combinations are unrepresentable.** Arity coupling (a dual-pivot partition
  needs a two-pivot selector) is baked into types, so the compiler can't emit a
  nonsense pairing.
- **Complexity/stability compose automatically.** Each factor declares its own Big-O;
  the outer algorithm's bound is computed from its parts at compile time.
- **Curate by query, not by code.** Want quicksort but *without* the last-element
  pivots? You write a set-difference in a query file; you don't touch the algorithm.

---

<a name="2-mental-model"></a>
## 2. The mental model: traits vs roles vs the compiler

This is the concept newcomers trip on, so it comes first.

There are **three** layers, and they are not the same thing:

| Layer | Lives in | What it is |
|---|---|---|
| **Trait** | Rust (`array_vis_bench_traits`) | The compile-time *contract* , `PartitionScheme`, `PivotSelector`, `SmallSort`. |
| **Role** | `.spec` files (strings) | A *semantic bucket* grouping impls for enumeration , `PartScheme`, `PartPivot`, `PartitionKind`. |
| **Component** | `.spec` files | A named binding of a Rust type to the role(s) it provides and the slots it needs. |

### A trait is the Rust contract

`PivotSelector` is a normal Rust trait. `FirstElement` implements it. rustc guarantees
that anything bound `Sel: PivotSelector` really has a `select` method.

### A role is a spec-level tag, and roles are finer-grained than traits

A **role** is just a string in a `.spec` file that groups components so a query can say
"fill this slot with anything of role X". Crucially, **one Rust trait is sliced into
several roles**, cut as finely as the queries need:

- The trait `PivotSelector` / `PivotInput` backs the roles `PartPivot` (single pivots),
  `PartPivotDual` (dual selectors like `CombinedSelector`), and family-specific ones
  like `BeapDualPivot`.
- The trait `PartitionScheme` backs `PartScheme` (pivot-consuming single partitions),
  `PartitionKind` (self-pivoting composites),
  `SelectPartition` (composites + heap-extract), and `HeapExtractPart`.

And **one component can provide several roles at once**. From
[quick.spec](../crates/families/quick_sort_lib/quick.spec):

```text
component pk_single
  type     Pivoted<{scheme}, {pivot}>
  label    {scheme}<{pivot}>
  provides PartitionKind SelectPartition     # <-- two roles
  category Partition
  uses     quick_sort_lib::Pivoted
  slot     scheme PartScheme
  slot     pivot  PartPivot
end
```

So: **slots bind by *role*, not by trait.** When you add a component, you choose which
role(s) it advertises; that , not its Rust trait alone , decides which slots it can
fill and therefore which algorithms it appears in.

### The compiler ties them together

At build time the compiler reads every `.spec` (the components + their roles), reads
every `.avbs` query (which slots to fill with which roles), enumerates the legal
combinations, and emits one concrete Rust type per combination. rustc then compiles
those types , so the role system is a *type-safe* enumerator: if a role lets a bad type
into a slot, the emitted code fails to compile.

---

<a name="3-anatomy-of-a-component"></a>
## 3. Anatomy of a component: Rust impl + `.spec` + `.avbs`

Every component you add is up to three artifacts. Here is the whole pipeline on one
page, using the real `LastElement` pivot.

### (a) The Rust impl , implement a role trait

A leaf lives in its own tiny crate and implements one trait. `FirstElement`
([crates/pivots/pivot_first/src/lib.rs](../crates/pivots/pivot_first/src/lib.rs)):

```rust
pub struct FirstElement;

impl PivotSelector for FirstElement {
    const NAME: &'static str = "first";
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(
        _arr: &[T],
        _logger: &mut U,
    ) -> usize {
        0
    }
}
```

Plus its **composable annotations** (see [section 6](#6-the-trait-catalog)) , the four `const`
impls that let the outer sort compute its Big-O:

```rust
impl HasTimeBounds for FirstElement {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for FirstElement { const SPACE: Complexity = Complexity::CONST; }
impl HasStability for FirstElement { const STABLE: bool = true; }
impl PivotQuality for FirstElement { const DEGENERATES: bool = true; }
```

### (b) The `.spec` declaration , bind the type to a role

A `.spec` fragment (owned by the same crate) names the component and says what role it
provides. [crates/pivots/pivot_last/pivot_last.spec](../crates/pivots/pivot_last/pivot_last.spec)
in full:

```text
component piv_last
  type     LastElement
  label    last
  provides PartPivot
  uses     pivot_last::LastElement
end
```

That is the entire second half of the job for a leaf. Five lines. A type that
several families slot under different roles is still one component. Its
`provides` line lists them all, as `pivot_first.spec` does with `PartPivot
HeapPiv BeapPiv`.

#### `.spec` keyword reference

| Keyword | Meaning |
|---|---|
| `component <name>` ... `end` | Declares one component. `<name>` is globally unique across the whole workspace (a duplicate is a build-time panic). |
| `type <expr>` | The Rust type, with `{hole}` placeholders for each slot. Leaves have no holes (`LastElement`); composites do (`Pivoted<{scheme}, {pivot}>`). |
| `label <template>` | Human-readable name, same `{hole}` templating. Instantiated into the `AlgorithmEntry.name` / picker path. |
| `provides <Role> [<Role>...]` | The role(s) this component advertises. This is what slots match against. |
| `uses <rust::path> [<path>...]` | Module paths spliced into the emitted `use` block so `type` resolves. |
| `slot <name> <Role> [<default>]` | A type parameter, filled by any component of `<Role>`. Optional `<default>` component if a query omits it. |
| `project <name> <Role>` | A *structural-only* parameter that does **not** appear in `type` , exists only so a query can refine on it (`Partition[pivot = ...]`). |
| `category <Cat>` | If present, this component is emitted as a first-class catalog entry (`Sort`, `Partition`, `QuickSelect`, ...). Omit for pure slot-fillers. |
| `menu <path>` | Picker sub-path beneath the category. |
| `max_input <n>` | Contract cap: inputs larger than `n` are out of contract (small sorts, or expensive exotic sorts). Becomes `AlgorithmEntry.max_input_size`. |
| `adaptive <bool>` | Per-family literal: faster on nearly-sorted input. |

### (c) The `.avbs` query , enumerate the combinations

A leaf usually needs no query , it's consumed by *other* components' queries. The query
lives with the **driver**. [quick.avbs](../crates/families/quick_sort_lib/quick.avbs):

```text
QuickSorts: SortingAlgorithm = SortingAlgorithm<CanonicalQuickSort<
    part = _ - {
        Pivoted<_, LastElement>,
        DualPivoted<CombinedSelector<LastElement, _>>,
        DualPivoted<CombinedSelector<_, LastElement>>
    },
    small_sort = _
>>
```

Read it as: *emit every `CanonicalQuickSort` as a `SortingAlgorithm`; fill `part` with
every `PartitionKind` **except** the last-element ones; fill `small_sort` with every
`SmallSort`.*

#### `.avbs` syntax reference

The grammar is `Name: Type = Value`.

- **`Name`** , the binding; each becomes its own solved query.
- **`Type`** , a role, or an emit **wrapper** like `SortingAlgorithm<...>` / `Selection<...>`.
  The wrapper marks the binding as emittable and sets the `AlgorithmEntry` category.
- **`Value`** , the constraint expression:

| Form | Meaning |
|---|---|
| `_` or `*` | Exhaustive: every component providing the slot's role. |
| `?` / `?N` / `?N@seed` | Random sample of 1 / N components (optionally seeded). |
| `name = _` | Named-argument fill of a specific slot. |
| `_ - { A, B }` | Set difference: exhaustive minus everything matching A or B. |
| `_ where ( A | B | C )` | Union: only these curated alternatives. |
| `SortingAlgorithm<X>` etc. | Emit wrapper. |
| `depth N` | Bound recursion depth for self-referential families ([section 9](#9-advanced)). |

Set-difference and union subtract/select by the **resolved type string**, so
`Pivoted<_, LastElement>` removes exactly the last-element single partitions regardless
of which component built them.

### (d) What gets emitted

The compiler solves each query into concrete types and emits, per type, a module like:

```rust
mod __entry_N {
    use quick_sort_lib::Pivoted;
    use partition_lomuto::LeftLeftPartition;
    use pivot_first::FirstElement;

    pub type Ty = Pivoted<LeftLeftPartition, FirstElement>;

    #[linkme::distributed_slice(ALGORITHMS)]
    static ENTRY: AlgorithmEntry = AlgorithmEntry {
        name: "LL<first>",
        category: Category::Partition,
        worst:   <Ty as HasTimeBounds>::WORST,   // inherited, composed
        stable:  <Ty as HasStability>::STABLE,
        max_input_size: None,
        run_with_input: /* fn ptr */,
        run_correctness: /* fn ptr */,
        // ...
    };
}
```

The `AlgorithmEntry` struct
([array_vis_bench_core/src/bench_registry.rs](../array_vis_bench_core/src/bench_registry.rs)):

```rust
pub struct AlgorithmEntry {
    pub name: &'static str,
    pub category: Category,
    pub worst: Complexity,
    pub best: Complexity,
    pub average: Complexity,
    pub space: Complexity,
    pub stable: bool,
    pub adaptive: bool,
    pub max_input_size: Option<usize>,
    pub run_with_input: fn(&str, &RunConfig, &mut dyn sort_logger::SortLogger<usize>),
    pub run_correctness: fn(),
}
```

---

<a name="4-the-logger-contract"></a>
## 4. The logger contract and instrumentation discipline

Every component takes a `logger: &mut U` where `U: ?Sized + SortLogger<T>`. This is not
optional plumbing , it is how the algorithm is *observed*.

### The iron rule

> **Every array read that a comparison depends on, and every array write, goes through
> the logger. No raw `arr[i] < arr[j]`, no raw `arr.swap(i, j)`.**

The visualiser only knows what the logger is told. A raw `arr.swap(i, j)` mutates the
array but emits no event, so the video silently desyncs from reality and the benchmark's
operation counts are wrong. Route everything through the trait:

```rust
if logger.cmp_gt(arr, i, ii) { logger.swap(arr, i, ii); }   // right: observed
if arr[i] > arr[ii] { arr.swap(i, ii); }                    // wrong: invisible
```

**Preferred API:** reach for the fused conditionals , `cond_swap_lt`, `cond_swap_gt`,
`swap` , over hand-rolling `cmp_gt_data` + `write`. They read better and emit the right
event pairing. `Size2SmallSort`'s entire body is one call:

```rust
fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) -> bool {
    if arr.len() == 2 { logger.cond_swap_gt(arr, 0, 1) } else { false }
}
```

### Why the signature is always `T: Ord + Copy, U: ?Sized + SortLogger<T>`

This exact shape appears on every role method. Two deliberate choices:

- **`U: ?Sized`** is what lets the *same* code run under both a monomorphic
  `&mut NoOpLogger` (the benchmark path , zero-cost, methods inline to nothing) and a
  `&mut dyn SortLogger<T>` (the visualiser path). Drop `?Sized` and the `dyn` path stops
  compiling. Always include it.
- **The trait's methods are non-generic** (parameterised only by the trait's own `T`,
  never a method-level `<S>`). That is what keeps `SortLogger<T>` **dyn-compatible**.
  If you ever extend the trait, keep new methods non-generic or you break every `dyn`
  call site.

### Array identity is the base pointer

The logger identifies an array by its base pointer (via the internal `arr_name!` macro).
Practical consequence: **pass the same slice object consistently.** If you reslice, the
visualiser treats it as a different array. This is why auxiliary buffers are allocated
*through* the logger and threaded around rather than reallocated.

### Auxiliary storage lifecycle

Allocations that grow with N are bracketed so the visualiser can show (and later retire)
the aux array:

```rust
let mut buf = logger.create_aux_arr(len);   // announces the array
// ... use buf, moving elements via logger.write_accross / write_data_u ...
logger.free_aux_arr(&buf);                   // announces its removal
```

For partition scratch there's a helper that owns this once for a whole sort,
[`with_partition_scratch`](../array_vis_bench_traits/src/role/partition.rs) , the driver
allocates `P::SCRATCH_LEN` cells one time and threads the same slice into every
`partition` call. Schemes with `SCRATCH_LEN == 0` get an empty slice and skip the aux
events entirely.

---

<a name="5-common-patterns"></a>
## 5. Common patterns

### 5.1 A leaf , implements a father trait, generic over nothing

The base case: a plain unit struct implementing one role trait.
`FirstElement` (above) and `NoSmallSort`
([crates/small_sorts/small_sort_basic/src/lib.rs](../crates/small_sorts/small_sort_basic/src/lib.rs)):

```rust
pub struct NoSmallSort;

impl SmallSort for NoSmallSort {
    const THRESHOLD: usize = 0;
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(_arr: &mut [T], _logger: &mut U) -> bool {
        unreachable!("NoSmallSort::sort should never be called (THRESHOLD = 0)")
    }
}
```

Note the `THRESHOLD = 0` sentinel: `NoSmallSort` exists purely as the "never cut over to
a small sort" choice in the small-sort axis. Leaves are cheap; make one per idea.

### 5.2 Generic over one component

A type that wraps one other component to adapt or extend it. `SmallSortAdapter<S>`
([array_vis_bench_traits/src/role/small_sort.rs](../array_vis_bench_traits/src/role/small_sort.rs))
lifts any `SmallSort` into a fixed-size `SetSizeSmallSort<N>`:

```rust
pub struct SmallSortAdapter<S: SmallSort>(PhantomData<S>);

impl<const N: usize, S: SmallSort> SetSizeSmallSort<N> for SmallSortAdapter<S> {
    #[inline(always)]
    fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T; N], logger: &mut U) {
        const { assert!(N <= S::THRESHOLD, "SmallSortAdapter<S>: N must be <= S::THRESHOLD"); }
        let _ = S::sort(arr, logger);
    }
}
```

The single type parameter `S` is threaded straight through to `S::sort`. The
`const { assert!(...) }` shows a nice trick: a bound that would be awkward in the type
system (`N <= S::THRESHOLD`) enforced as a compile-time assertion.

### 5.3 Generic over multiple components

The composer pattern , several axes fused into one type. `Pivoted<Scheme, Sel>`
([crates/families/quick_sort_lib/src/pivoted.rs](../crates/families/quick_sort_lib/src/pivoted.rs))
takes a partition scheme *and* a pivot selector:

```rust
pub struct Pivoted<Scheme, Sel>(PhantomData<(Scheme, Sel)>);

impl<Scheme: PartitionScheme, Sel: PivotInput> PartitionScheme for Pivoted<Scheme, Sel> {
    const NAME: &'static str = Scheme::NAME;
    const N_PIVOTS: usize = 0;                       // self-pivoting
    const SCRATCH_LEN: usize = Scheme::SCRATCH_LEN;

    fn partition<T, U, V>(arr: &mut [T], logger: &mut U, _pivots: &[usize],
                          scratch: &mut [usize], visitor: &mut V)
    where T: Ord + Copy, U: ?Sized + SortLogger<T>, V: PartitionVisitor {
        let mut pivots = [0usize; 2];
        Sel::pick(arr, logger, &mut pivots);         // pick with the selector ...
        Scheme::partition::<T, U, V>(arr, logger, &pivots[..Sel::N], scratch, visitor); // ... then delegate
    }
}
```

Three-parameter composers are common too , e.g.
`QuickHeapSort<A: Arity, DH: DeepHeapify, SS: SmallSort>`
([crates/families/quick_heap_sort_lib/src/quick_heap_sort.rs](../crates/families/quick_heap_sort_lib/src/quick_heap_sort.rs)),
which threads all three parameters into its `recurse` helper.

### 5.4 Two families, one father trait: specialised vs composed

**This is the pattern that makes the system expressive.** Two completely different
type-families can both satisfy the *same* father trait, and a query can mix them freely
in the same slot.

Concretely, a dual-pivot partition can be either a hand-written monolith *or* a
composition of two single-pivot selectors , and both are `PartitionScheme`.

**Family A , the specialised monolith.** `DualPivotPartition`
([crates/families/quick_sort_lib/src/yaroslavskiy.rs](../crates/families/quick_sort_lib/src/yaroslavskiy.rs))
implements Yaroslavskiy's two-pointer scan directly, taking two pivots:

```rust
pub struct DualPivotPartition;

impl PartitionScheme for DualPivotPartition {
    const NAME: &'static str = "dual pivot";
    const N_PIVOTS: usize = 2;                        // caller supplies two pivots
    fn partition<T, U, V>(arr: &mut [T], logger: &mut U, pivots: &[usize],
                          _scratch: &mut [usize], visitor: &mut V) { /* Yaroslavskiy */ }
}
```

**Family B , the composition.** Take a *single*-pivot partition, wrap it in
`Pivoted`, and feed it a `CombinedSelector<V1, V2>` that fuses two single selectors into
a dual `PivotInput` with `N = 2`
([crates/families/quick_sort_lib/src/pivot_selectors.rs](../crates/families/quick_sort_lib/src/pivot_selectors.rs)):

```rust
pub struct CombinedSelector<V1, V2>(PhantomData<(V1, V2)>);

impl<V1: PivotSelector, V2: PivotSelector> DualPivotSelector for CombinedSelector<V1, V2> {
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], logger: &mut U) -> (usize, usize) {
        let p1 = V1::select(arr, logger);
        let p2 = if arr.len() > 1 { 1 + V2::select(&arr[1..], logger) } else { 0 };
        (p1, p2)
    }
}
impl<V1: PivotSelector, V2: PivotSelector> PivotInput for CombinedSelector<V1, V2> {
    const N: usize = 2;
    fn pick<...>(arr, logger, out) { let (p1, p2) = Self::select(arr, logger); out[0] = p1; out[1] = p2; }
}
```

Now `DualPivotPartition` and `Pivoted<SomeScheme, CombinedSelector<First, Last>>` are
**both** `PartitionScheme`, arriving there by opposite routes:

| | Family A: `DualPivotPartition` | Family B: `Pivoted<S, CombinedSelector<V1,V2>>` |
|---|---|---|
| `N_PIVOTS` | `2` (driver supplies pivots) | `0` (picks its own) |
| Pivot logic | hard-coded inside `partition` | delegated to `V1`/`V2` selectors |
| Reuse | none , bespoke to Yaroslavskiy | any scheme x any two selectors |
| When to write it | the algorithm is genuinely different | you're recombining existing parts |

The rule of thumb: **write a monolith only when the algorithm is genuinely
different**; otherwise compose. In the `.spec`, the specialised form is baked
into the composite `pk_dual`, whose `DualPivoted<{pivot}>` feeds the same
`PartitionKind` population as the composed `pk_single`.

### 5.5 Report-don't-return: the visitor contract

A `PartitionScheme` does **not** return its sub-ranges. It *reports* each unsorted region
to a `PartitionVisitor` and lets the caller decide what to do with it:

```rust
pub trait PartitionVisitor {
    fn unsorted(&mut self, range: Range<usize>);
}
```

Sorted regions (the pivots, equal-pinned runs) are the **implicit gaps** between
`unsorted` calls. This decouples the partition from the recursion strategy: the same
partition works under a recursive driver, an iterative one, or the standalone
`Category::Partition` correctness harness, which plugs in a tiny visitor that just
captures boundaries
([array_vis_bench_traits/src/role/partition.rs](../array_vis_bench_traits/src/role/partition.rs)):

```rust
struct BoundsVisitor { left_end: usize, right_start: usize, n: u8 }
impl PartitionVisitor for BoundsVisitor {
    fn unsorted(&mut self, r: Range<usize>) {
        if self.n == 0 { self.left_end = r.end; }
        else if self.n == 1 { self.right_start = r.start; }
        self.n += 1;
    }
}
```

When you write a partition, **announce every region you leave unsorted** and swap
everything through the logger; do not try to sort or return them.

### 5.6 Adapters, blanket impls, and const-threading

The cross-product stays small because of *lifting* , blanket impls that make a narrow
trait satisfy a wider one for free. The key one is in
[pivot.rs](../array_vis_bench_traits/src/role/pivot.rs): every single `PivotSelector`
becomes a `PivotInput` with `N = 1` automatically ,

```rust
impl<V: PivotSelector> PivotInput for V {
    const N: usize = 1;
    fn pick<...>(arr, logger, out) { out[0] = V::select(arr, logger); }
}
```

, so a single selector and a `CombinedSelector` (`N = 2`) are interchangeable wherever a
`PivotInput` is wanted, without writing an adapter per selector. Associated `const`s ride
along the composition: `Pivoted`'s `NAME` is just `Scheme::NAME`, and its Big-O consts
are computed from its parts (next section).

---

<a name="6-the-trait-catalog"></a>
## 6. The trait catalog

The traits you'll touch fall into three groups. All live in the pure-trait crate
`array_vis_bench_traits` (re-exported from its crate root; role traits under
`array_vis_bench_traits::role`).

### 6.1 Algorithmic role traits , the behaviour you implement

Every method is generic over `<T: Ord + Copy, U: ?Sized + SortLogger<T>>`.

| Trait | Key members | Role of the impl |
|---|---|---|
| `PartitionScheme` | `const NAME`, `const N_PIVOTS`, `const SCRATCH_LEN = 0`, `fn partition(arr, logger, pivots, scratch, visitor)` | One partition step; reports unsorted regions to a visitor. |
| `PivotSelector` | `const NAME`, `fn select(arr, logger) -> usize` | Choose one pivot index. |
| `DualPivotSelector` | `fn select(arr, logger) -> (usize, usize)` | Choose two pivot indices. |
| `PivotInput` | `const N`, `fn pick(arr, logger, out)` | Unified 0/1/2-pivot source; blanket-lifts `PivotSelector` (`N=1`); `NoPivot` is `N=0`. |
| `SmallSort` | `const THRESHOLD`, `fn sort(arr, logger) -> bool` | Sort any `len <= THRESHOLD`; returns whether it mutated. |
| `NonTrivialSmallSort` | (marker, `: SmallSort`) | `THRESHOLD > 1`. |
| `InsertionStrategy` | `fn insert_one(arr, i, logger) -> bool` | Place one element of a sorted prefix. |
| `SetSizeSmallSort<N>` | `fn sort(arr: &mut [T; N], logger)` | Fixed-size sort; `SmallSortAdapter<S>` lifts a `SmallSort` into it. |
| `DeferredSmallSort` | `const THRESHOLD`, `fn final_pass(arr, logger)` | One whole-array cleanup pass after a deferred quicksort. |
| `Rotation` | `const NAME`, `fn scratch_size(n) -> usize`, `fn rotate(arr, split_ind, scratch, logger)` | Rotate a slice; in-place or buffered. |
| `QuickSelect` | `fn select(arr, logger, target)` | Place the `target`-th order statistic at index `target`. |
| `Scratchspace` | `const NAME`, `fn aux_size(n)`, `acquire`/`release`/`park`/`place`/`block_swap`/`relocate`/`roll` | Bounded-buffer contract for block-merge sorts (WikiSort/GrailSort). |
| `Auxiliary` | `const NAME`, `const MERGE_WORST`, `const MERGE_SPACE`, `fn scratch_size(n)` | How much scratch a merge is handed (-> in-place vs buffered). |

### 6.2 Annotation traits , the Big-O that composes itself

These carry no behaviour, only `const`s, and default to "unknown". The compiler reads
them off the final concrete type to fill `AlgorithmEntry`'s complexity fields.
([composable.rs](../array_vis_bench_traits/src/composable.rs))

| Trait | Const | Default |
|---|---|---|
| `HasTimeBounds` | `WORST`, `BEST`, `AVERAGE: Complexity` | `UNKNOWN` (best/avg fall back to worst) |
| `HasSpace` | `SPACE: Complexity` | `UNKNOWN` |
| `HasStability` | `STABLE: bool` | `false` |
| `PivotQuality` | `DEGENERATES: bool` | `true` (assume worst) |

`Complexity` ([complexity.rs](../array_vis_bench_traits/src/complexity.rs)) is a
const-evaluable Big-O class with pure `const fn` `product` (exponents add) and `sum`
(max). A composer combines its parts' bounds at compile time , e.g. `Pivoted` sums the
scheme's and selector's costs:

```rust
impl<Scheme, Sel> HasTimeBounds for Pivoted<Scheme, Sel>
where Scheme: PartitionScheme + HasTimeBounds, Sel: PivotInput + HasTimeBounds {
    const WORST:   Complexity = Complexity::sum(Scheme::WORST,   Sel::WORST);
    const BEST:    Complexity = Complexity::sum(Scheme::BEST,    Sel::BEST);
    const AVERAGE: Complexity = Complexity::sum(Scheme::AVERAGE, Sel::AVERAGE);
}
```

**When you write a leaf, add these four impls.** If you skip them the type falls back to
`O(?)`/non-stable and pollutes every composition it enters (`UNKNOWN` dominates). Use the
predefined constants (`Complexity::CONST`, `N_LOG_N`, `N_SQUARED`, ...).

### 6.3 Dispatch wrappers and the legacy trait

- **Emit wrappers** (`SortingAlgorithm<X>`, `Selection<X>`, `RotationAlg<X>`, ...) are not
  Rust traits you implement , they're `.avbs` markers that tag a solved type for emission
  and pick its `Category`.
- `SortAlgo<T, U>` in [sort_traits.rs](../array_vis_bench_traits/src/sort_traits.rs) is a
  legacy object-style dispatch trait. New components prefer inherent `fn sort` methods and
  the spec pipeline; you generally won't implement `SortAlgo` for new work.

### 6.4 The logger trait

`SortLogger<T: Copy + Ord>` ([sort_logger/src/sort_logger.rs](../sort_logger/src/sort_logger.rs))
is the instrumentation surface , ~50 methods, all with default bodies that emit a
`SortLog<T>` event, all dyn-compatible (see [section 4](#4-the-logger-contract)). The groups you
reach for:

- **Compare:** `cmp_eq/neq/lt/le/gt/ge` (in-array), `*_data` (vs a value), `*_accross` (between arrays).
- **Move:** `write` (a[i]=a[j]), `write_data` (a[i]=v), `swap`, `cond_swap_lt/le/ge/gt`, `write_accross`.
- **Compound:** `reverse`, `block_swap`, `shift_insert`, `copy_range`.
- **Aux arrays:** `create_aux_arr`/`free_aux_arr` (usize), `create_aux_arr_t`/`copy_aux_arr_t`/`free_aux_arr_t` (element-typed), `*_u8` family; `write_data_u`/`write_data_u8` to mutate them observably.
- **Markers:** `mark`, `mark_mssg`.

Two impls back it: `NoOpLogger` (benchmark; every method a no-op) and
`VisualizerLogger<T>` (records events for the renderer). Adding a new integer width for an
aux array is one `int_aux_family!(...)` line , the macro generates the four non-generic
methods so dyn-compatibility is preserved.

---

<a name="7-getting-it-into-the-catalog"></a>
## 7. Getting it into the catalog: crates, discovery, build, tests

### Crate topology , where your file goes

- **`array_vis_bench_traits`** , the shared root: all role traits, `Complexity`, the
  annotation traits. Depends on nothing but `sort_logger`.
- **Leaf crates** , `crates/pivots/*`, `crates/partitions/*`, `crates/small_sorts/*`,
  `crates/rotations/*`, ... Each is tiny and depends on **only** `array_vis_bench_traits`
  + `sort_logger`. This is deliberate: a leaf must not drag the whole tree in.
- **Family crates** , `crates/families/*` , hold the composers and drivers (`Pivoted`,
  `CanonicalQuickSort`, `QuickHeapSort`, `CombinedSelector`). They depend on the leaves
  they combine.

Put a new single-purpose thing in its own leaf crate; put a composer with the family
whose driver consumes it.

### Discovery , the Cargo.toml handshake

`spec_catalog`'s `build.rs` walks the dependency closure and picks up any crate that
self-describes in its manifest. That's the only wiring step
([crates/pivots/pivot_first/Cargo.toml](../crates/pivots/pivot_first/Cargo.toml)):

```toml
[package.metadata.array_vis_bench]
spec  = "pivot_first.spec"      # relative to the crate root
# query = "quick.avbs"          # drivers also list a query
```

Then make sure `spec_catalog` (transitively) depends on your crate, or its `.spec`
won't be in the closure. A misspelled key or a missing dependency is a **build-time
panic**, not a silent no-op , the pipeline fails fast so a family can't quietly vanish.

### The build pipeline, briefly

`build.rs` -> **gather** every `.spec`/`.avbs` in the closure -> **merge** all `.spec`
fragments into one global registry (duplicate component name => panic) -> **solve** each
`.avbs` binding against the registry (a query that solves to **zero rows** => panic) ->
**emit** `catalog_entries.rs`, one `#[distributed_slice(ALGORITHMS)]` entry per solved
type. Nothing is hand-registered; the linkme slice is assembled at link time.

### Correctness expectations

Each emitted entry carries a `run_correctness` fn pointer, exercised by the workspace
test suite (`cargo test`). The battery runs category-appropriate inputs under
`NoOpLogger` and verifies the result. Two things you must respect:

- **`max_input`** , if your component is only correct/feasible up to some size, declare
  it. The correctness harness and the picker both honour `max_input_size`; a global test
  `N` that ignores per-entry caps will OOM on an exotic sort.
- **Slow or nondeterministic sorts** , very slow sorts register a cap via
  `SORT_TEST_CAPS`; sorts whose trace is intentionally nondeterministic opt out of the
  determinism check. Defaults are "no cap" and "deterministic" , you only act if your
  sort is unusual.

A leaf's own crate should carry at least one unit test that its `sort`/`select`/`rotate`
actually sorts/selects/rotates. Keep it minimal , one property-style check is plenty.

---

<a name="8-worked-example"></a>
## 8. Worked example: add a pivot selector end to end

Goal: add a "penultimate element" pivot and watch it appear across the whole quicksort /
quick-select catalog. Total new code: one small crate.

**1. Create the leaf crate** `crates/pivots/pivot_penultimate/`.

`Cargo.toml`:

```toml
[package]
name = "pivot_penultimate"
version = "0.1.0"
edition = "2021"

[lib]
name = "pivot_penultimate"
path = "src/lib.rs"

[dependencies]
array_vis_bench_traits = { path = "../../../array_vis_bench_traits" }
sort_logger            = { path = "../../../sort_logger" }

[package.metadata.array_vis_bench]
spec = "pivot_penultimate.spec"
```

**2. Implement the trait + annotations** in `src/lib.rs`:

```rust
use array_vis_bench_traits::{
    Complexity, HasSpace, HasStability, HasTimeBounds, PivotQuality, PivotSelector,
};
use sort_logger::SortLogger;

/// Pivot = the second-to-last element (last-1), or 0 for tiny arrays.
pub struct Penultimate;

impl PivotSelector for Penultimate {
    const NAME: &'static str = "penultimate";
    fn select<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &[T], _logger: &mut U) -> usize {
        arr.len().saturating_sub(2)   // positional: no comparisons, no logging
    }
}

// Positional selector: constant work, no space, order-preserving, can degenerate
// on adversarial input (same story as FirstElement / LastElement).
impl HasTimeBounds for Penultimate {
    const WORST: Complexity = Complexity::CONST;
    const BEST: Complexity = Complexity::CONST;
    const AVERAGE: Complexity = Complexity::CONST;
}
impl HasSpace for Penultimate { const SPACE: Complexity = Complexity::CONST; }
impl HasStability for Penultimate { const STABLE: bool = true; }
impl PivotQuality for Penultimate { const DEGENERATES: bool = true; }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn picks_last_minus_one() {
        assert_eq!(Penultimate::select(&[9, 8, 7, 6], &mut ()), 2);
        assert_eq!(Penultimate::select(&[1], &mut ()), 0);   // saturates
    }
}
```

(`&mut ()` works as a no-op logger in tests , `()` implements `SortLogger`.)

**3. Declare the component** in `pivot_penultimate.spec`:

```text
component piv_penultimate
  type     Penultimate
  label    penultimate
  provides PartPivot
  uses     pivot_penultimate::Penultimate
end
```

**4. Add the crate to the closure** , add `pivot_penultimate` as a dependency of
whatever crate pulls the pivot leaves into `spec_catalog` (the same place `pivot_first`,
`pivot_last` are listed), and to the workspace members if needed.

**5. Build.** `cargo build` re-runs `spec_catalog/build.rs`. Because `piv_penultimate`
provides `PartPivot`, it's now a legal filler for `pk_single`'s `pivot` slot and for
`combined`'s `a`/`b` sub-slots. It flows automatically into:

- every single-pivot `PartitionKind` (`Pivoted<scheme, penultimate>`) -> all quicksorts,
- every dual composite that pairs it (`combined<penultimate, ...>`),
- quick-select and the standalone Partition-kind entries.

To also offer it to the heap and beap quick builds, add `HeapPiv` and `BeapPiv`
to the same `provides` line.

You wrote one struct and five lines of spec; the compiler produced dozens of new
catalog entries, each with correct inherited Big-O. If you *didn't* want it in some
consumer, you'd subtract it there with `_ - { Pivoted<_, Penultimate> }` , in the query,
not the code.

---

<a name="9-advanced"></a>
## 9. Advanced

### Recursion bounding for self-referential families

Some families are genuinely recursive , a partition that nests a quick-select that
nests a partition. Left alone, the enumerator would unfold that cycle forever. Two tools:

- **`depth N`** (an `.avbs` directive) caps how deep the solver unfolds a recursive role.
- **`finite(_)`** wraps a hole and keeps only the trees where **no component nests inside
  itself** (an *occurs check*). A cyclic role wrapped in `finite` unfolds exactly once,
  yielding the principled base variants instead of an unbounded family.

If you introduce a component whose slot role can (transitively) be filled by the
component itself, you need one of these or the build won't terminate. See the
set-quick-select heap family for a worked-in-tree use of `finite` + `depth 12`.

### Curate in the query, not in the algorithm

A recurring project principle: **when one variant of a generated cross-product
misbehaves, remove it from the query , don't special-case the algorithm.** The set
operators exist for exactly this:

```text
part = _ - { Pivoted<_, LastElement> }        # drop the bad axis value
part = _ where ( ... | ... )                       # or hand-pick the good ones
```

Patching a driver to paper over one degenerate combination adds a branch that every
other combination pays for and that the next reader has to decode. Dropping the axis
value keeps each component honest and the drivers clean. Reach for `_ - {...}` / `where`
first.

---

## Quick checklist for a new leaf

- [ ] New crate under `crates/<axis>/<name>/`, depends only on `array_vis_bench_traits` + `sort_logger`.
- [ ] `impl <RoleTrait> for <YourType>` with the standard `<T: Ord + Copy, U: ?Sized + SortLogger<T>>` signature.
- [ ] All array access routed through `logger` (prefer `cond_swap_*`).
- [ ] The four annotation impls (`HasTimeBounds`, `HasSpace`, `HasStability`, and `PivotQuality` if it's a pivot).
- [ ] A `<name>.spec` fragment: unique `component`, correct `provides <Role>`, `uses <rust::path>`.
- [ ] `[package.metadata.array_vis_bench] spec = "<name>.spec"` in `Cargo.toml`.
- [ ] Crate is in `spec_catalog`'s dependency closure.
- [ ] One unit test proving it does its job; `max_input` if it's size-limited.
- [ ] `cargo build` (regenerates the catalog) then `cargo test`.
