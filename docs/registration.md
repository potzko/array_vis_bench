# Algorithm Registration System

How every algorithm — sort, rotation, partition, merge, quick-select, small-sort — announces itself to the framework without a central list.

## The single source of truth: `bench_registry::ALGORITHMS`

Everything ends up as one `AlgorithmEntry` in a `linkme` distributed-slice (`bench_registry::ALGORITHMS`). The visualiser, the speed-test binary, the Criterion-free bench harness, and the correctness test suite all iterate that single slice.

```rust
pub struct AlgorithmEntry {
    pub name: &'static str,
    pub category: Category,        // Sort | Rotation | Partition | Merge | SmallSort | QuickSelect
    pub big_o: &'static str,
    pub stable: bool,              // sort-specific; ignored for other categories
    pub max_input_size: Option<usize>,
    pub run_with_input:  fn(&str, &RunConfig, &mut dyn SortLogger<usize>),
    pub run_correctness: fn(),
}

#[distributed_slice]
pub static ALGORITHMS: [AlgorithmEntry] = [..];
```

`run_with_input` is the harness contract: given a registered input name, emit the initial-state events and run the algorithm against it. `run_correctness` invokes the category's correctness battery (random + structured inputs, sortedness + permutation + category-specific shape checks).

## Mechanism 1: `combo_codegen::family!`

The preferred path for new families. Build-script driven: every concrete component (e.g. `Lomuto`, `MedianOfThree`, `InsertionSmallSort<32>`) annotates itself with `combo_codegen::component!(Role, Type, "label")`, and every family annotates its generic-parameter slots with `combo_codegen::family!`. The build script scans the source tree, computes the cross-product, and writes one `AlgorithmEntry` per leaf to `$OUT_DIR/<family>_combinations.rs`.

```rust
combo_codegen::family!(
    type = QuickSort<{P}, {V}, {SS}>,
    uses = [
        "crate::sorts::quick_sorts::quick_sort::QuickSort",
        "crate::sorts::quick_sorts::partitions::{Lomuto, Hoare, ThreeWay, Block}",
        "crate::sorts::quick_sorts::pivot_selectors::{FirstElement, MiddleElement, MedianOfThree, Ninther}",
        "crate::utils::small_sort::{NoSmallSort, InsertionSmallSort, Size2SmallSort}",
        "crate::utils::small_sort::{LinearInsertion, BinaryInsertion}",
    ],
    P:  Partition,
    V:  PivotSelector,
    SS: SmallSort,
    name        = "quick sort",
    big_o       = "O(N log N)",
    stable      = false,
    direct_sort = true,
    path        = ["quick sorts", "{P}", "{V}", "{SS}"],
);
```

Adding a new pivot selector or partition scheme costs nothing in the family file: declare the type, add a `combo_codegen::component!` next to it, and every family that takes that slot picks it up on the next build.

## Mechanism 2: `sort_registry_macro::sort_family!`

The original, lighter-weight macro. Useful for single-leaf sorts or self-contained variant trees that don't need cross-family component scanning. Same `AlgorithmEntry` shape; same menu integration. See `src/sorts/bubble_sorts/bubble_sort.rs` for the minimal form.

## Mechanism 3: per-category `register_*!` macros

Non-sort algorithms use sibling macros that produce `AlgorithmEntry` with the matching `Category::*`:

| Category | Macro | Lives in |
|---|---|---|
| `Rotation` | `register_rotation!` | `src/utils/rotation/` |
| `Partition` | `register_partition!` | `src/sorts/quick_sorts/partitions_standalone.rs` |
| `Merge` | `register_merge!` / `register_aux_merge!` | `src/sorts/merge_sorts/standalone_registry.rs` |
| `QuickSelect` | `register_quick_select_single!` / `_dual!` | `src/sorts/quick_selects/standalone_registry.rs` |
| `SmallSort` | `register_small_sort!` | `src/utils/small_sort.rs` |

Each emits one `AlgorithmEntry` per concrete leaf, the same way `family!` does for sorts.

## What gets registered

Every leaf in any of the three mechanisms gets:

1. **A `linkme` `AlgorithmEntry`** in `bench_registry::ALGORITHMS` (the shared registry).
2. **A navigation-tree path** in `sort_registry_core::SORT_ENTRIES`, populated by a per-leaf `#[ctor::ctor]` that calls `register_sort_path`.
3. **An optional `register_test_cap!`** entry in `SORT_TEST_CAPS` if the algorithm caps random-input size for the correctness battery.

Validation runs in `bench_registry::validate_at_startup` before anything else: duplicate names, duplicate tree paths, missing or multiple primary inputs all panic at process start.

## Where inputs come from

Each category has its own `SortInputEntry` / `RotationInputEntry` / etc. slice. Inputs register themselves the same way algorithms do, and exactly one entry per category is marked `primary: true` (the harness's default when no input is named). See `src/inputs.rs`.
