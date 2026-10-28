# Trait system

Three groups of traits make the composition work: role traits that leaves
implement, annotation traits that carry complexity and stability as consts,
and the logger every body writes to. Together they are the calling
convention the compiler emits against.

## The sort shape

A sort exposes an inherent method, generic over the element and the logger:

```rust
impl<P: PartitionScheme, SS: SmallSort> CanonicalQuickSort<P, SS> {
    pub fn sort<T: Ord + Copy, U: ?Sized + SortLogger<T>>(arr: &mut [T], logger: &mut U) { ... }
}
```

A driver type also joins the `DirectSort` role with one line beside the impl,
`direct_sort!(CanonicalQuickSort<P, SS>; P: PartitionScheme, SS: SmallSort)`,
which forwards to the inherent method. The emitted entry runs the inherent
method, and `SortSuite` reaches it through the role to run the correctness
batteries, like every other kind's suite.

`U: ?Sized` is what lets one body serve both the monomorphic benchmark path
(`&mut NoOpLogger`) and the visualiser path (`&mut dyn SortLogger<T>`). Drop
it and the dyn path stops compiling.

## Role traits

All live in `array_vis_bench_traits/src/role/`. Every method takes the
`<T: Ord + Copy, U: ?Sized + SortLogger<T>>` pair.

| trait | key members | what an impl is |
|---|---|---|
| `PartitionScheme` | `NAME`, `N_PIVOTS`, `SCRATCH_LEN`, `partition(arr, logger, pivots, scratch, visitor)` | one partition step; reports unsorted ranges to a `PartitionVisitor` |
| `PartitionVisitor` | `unsorted(range)` | the driver side of the report, never return, contract |
| `PivotSelector` | `NAME`, `select(arr, logger) -> usize` | choose one pivot index |
| `DualPivotSelector` | `select(arr, logger) -> (usize, usize)` | choose two |
| `PivotInput` | `N`, `pick(arr, logger, out)` | 0, 1 or 2 pivot source; every `PivotSelector` is lifted to `N = 1` by a blanket impl, `NoPivot` is `N = 0` |
| `SmallSort` | `THRESHOLD`, `sort(arr, logger) -> bool` | sort any slice up to the threshold; `NoSmallSort` has threshold 0 |
| `NonTrivialSmallSort` | marker | threshold above 1 |
| `InsertionStrategy` | `insert_one(arr, i, logger) -> bool` | place one element of a sorted prefix |
| `SetSizeSmallSort<N>` | `sort(arr: &mut [T; N], logger)` | fixed size sort; `SmallSortAdapter<S>` lifts a `SmallSort` |
| `DeferredSmallSort` | `THRESHOLD`, `final_pass(arr, logger)` | one whole array cleanup after a deferred quicksort |
| `Rotation` | `NAME`, `scratch_size(n)`, `rotate(arr, split, scratch, logger)` | rotate a slice, in place or with a declared scratch |
| `QuickSelect` | `select(arr, logger, target)` | place the target order statistic |
| `DirectSort` | `sort(arr, logger)` | sort a whole array; each driver type forwards to its inherent `sort` with `direct_sort!` |
| `RotationMerge` | `scratch_size(n)`, `merge(arr, mid, scratch, logger)` | merge two adjacent sorted runs using a rotation |
| `Auxiliary` | `NAME`, `MERGE_WORST`, `MERGE_SPACE`, `scratch_size(n)`, `internal_size(n)` | how much scratch a merge is handed, external and carved from the array |
| `Keys` | `NAME`, `MERGE_WORST`, `MERGE_SPACE`, `key_count(blocks)`, `extract`, `tag_block`, `find_block`, `restore` | the key region that tags the blocks of a block merge, the movement imitation buffer |
| `Scratchspace` | `NAME`, `aux_size(n)`, `acquire`, `release`, `park`, `place`, `block_swap`, `relocate`, `roll` | a bounded buffer contract whose operations all conserve the multiset |
| `ScratchspaceRun` | `run(list, aux, logger)` | a scripted workflow over a `Scratchspace`, netting to identity |

Family local traits stay with their only consumer: `GapSequence` in
`shell_sort_lib`, `BranchingStrategy` and `RodMerge` in `rod_sort_lib`,
`GapDistribution` in
`gap_distribution_lib` (it needs `rand`), `ReverseStorage` in
`reverse_storage_lib` (it writes through the logger), and the heap layers
(`Heap`, `HeapLayout`, `Arity`, `Direction`, `DeepHeapify`, `HeapPartition`)
in `heap_sort_lib`.

### Report, do not return

A partition never returns its sub ranges. It announces each unsorted region
to the visitor and the caller decides what to do. Sorted regions are the
gaps between announcements. The same partition therefore runs under a
recursive driver, an iterative one, a quick select, and the standalone
partition battery, which plugs in a visitor that only records boundaries.

### Scratch through the logger

A scheme that needs scratch declares `SCRATCH_LEN`; `with_partition_scratch`
allocates it once per sort through the logger and threads the same slice into
every call. Rotations do the same with `scratch_size`. The visualiser can then
draw the buffer next to the array.

## Annotation traits

In `array_vis_bench_traits/src/composable.rs`. They carry consts only and
default to unknown. The emit reads them off the final concrete type.

| trait | consts | default |
|---|---|---|
| `HasTimeBounds` | `WORST`, `BEST`, `AVERAGE: Complexity` | unknown; best and average fall back to worst |
| `HasSpace` | `SPACE: Complexity` | unknown |
| `HasStability` | `STABLE: bool` | false |
| `PivotQuality` | `DEGENERATES: bool` | true |

`Complexity` (`complexity.rs`) is a const evaluable Big-O class with named
constants (`CONST`, `LOG_N`, `SQRT_N`, `N_LOG_N`, `N_SQUARED`, `CUBIC`,
`EXPONENTIAL`, `FACTORIAL`, `UNKNOWN`, among others) and two const functions:
`product` adds exponents, `sum` takes the maximum. A composer combines its
parts at compile time:

```rust
impl<Scheme, Sel> HasTimeBounds for Pivoted<Scheme, Sel>
where Scheme: PartitionScheme + HasTimeBounds, Sel: PivotInput + HasTimeBounds {
    const WORST: Complexity = Complexity::sum(Scheme::WORST, Sel::WORST);
    ...
}
```

Unknown dominates, so a leaf that skips its four impls pollutes every
composition it enters.

## The logger

`SortLogger<T>` in `sort_logger/src/sort_logger.rs` has one required method,
`log(SortLog<T>)`, and about fifty defaulted methods built on it: compares
(`cmp_lt`, `cmp_le_accross`, and friends), moves (`write`, `write_data`,
`swap`, `cond_swap_lt`, `write_accross`), compounds (`reverse`, `block_swap`,
`shift_insert`, `copy_range`), auxiliary arrays (`create_aux_arr_t`,
`free_aux_arr_t`, integer width variants), markers, and regions
(`hold_region` and `release_region` say that a sub range of an array is the
buffer or the key region of the sort for a while, which the renderer shades).
Every method is non
generic in anything but the trait's `T`, which keeps the trait dyn compatible.
The methods that need a second element type are gated `where Self: Sized`
and stay out of the vtable.

Two impls: `NoOpLogger`, whose methods compile away, and
`VisualizerLogger<T>`, which appends every event to a vector. `()` is a
convenient no-op logger in unit tests.

## Consumer side traits

`array_vis_bench_core::mains::Main` is a consumer over a set of entries
(visualiser, correctness, benchmark).
`array_vis_bench_core::suites::CorrectnessSuite` is the per kind correctness
battery, one impl per role trait backed kind, and is what the emit wires each
entry's `run_correctness` pointer to.
