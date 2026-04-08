# quick_sorts

Quick sort variants — different pivot selection and partitioning strategies.

## How quick sort works

Quick sort picks a *pivot* element, *partitions* the array so that everything less than the pivot is on the left and everything greater is on the right, then recursively sorts each side. The choice of pivot and partitioning scheme affects worst-case behaviour, constant factors, and stability.

## Variants

### Partition strategies

- **Left-right pointers, static pivot** (`quick_sort_left_right_pointers_static_pivot.rs`) — the default. Two pointers walk inward from both ends, swapping out-of-place pairs. The pivot stays at its original position until the partition is complete.
- **Left-right pointers, moving pivot** (`quick_sort_left_right_pointers_moving_pivot.rs`) — same two-pointer scheme but the pivot moves during partitioning.
- **Left-left pointers** (`quick_sort_left_left_pointers.rs`) — Lomuto-style: a single forward scan with a boundary pointer. Simpler but more swaps on average.
- **Left-left pointers optimised** (`quick_sort_left_left_pointers_optimised.rs`) — Lomuto with reduced swap count.
- **Left-right pivot optimised** (`quick_sort_left_right_pivot_optimised.rs`) — Hoare-style with pivot-selection optimisations.

### Pivot strategies

- **Median-of-three** (`midian_pivot_quick_sort.rs`) — picks the median of the first, middle, and last elements to avoid worst-case O(N^2) on sorted input.

### Other variants

- **Iterative quick sort** (`iterative_quick_sort.rs`) — replaces recursion with an explicit stack. Same algorithm, no stack overflow risk on large arrays.
- **Generic quick sort** (`generic_quick_sort.rs`) — parameterised over pluggable pivot and partition strategy traits. Used for combinatoric variant generation.
- **Pivot strategies** (`pivot_strategies.rs`) — trait + implementations for pivot selection.
- **Partition strategies** (`partition_strategies.rs`) — trait + implementations for partitioning.
- **Strategy registry** (`strategy_registry.rs`) — registration wiring for generic variants.
- **Auto register** (`auto_register.rs`) — automatic registration of combinations.

## Status

Not yet migrated to the `sort_family!` system. Compiled but commented out of the active dispatch in `sorts/mod.rs`.
