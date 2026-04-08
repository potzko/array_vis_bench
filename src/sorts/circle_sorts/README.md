# circle_sorts

Circle sort variants — a comparison sort that compares elements from opposite ends of a range, working inward like a circle.

## How circle sort works

A single circle-sort *pass* over `[start, end]` compares the outermost pair (`arr[start]` vs `arr[end]`), then the next pair inward, and so on, swapping whenever the right element is smaller. After reaching the middle, it has performed one "circle" of comparisons. The full sort repeats passes until no swaps occur.

Circle sort is not widely known but produces distinctive visual patterns and has interesting theoretical properties.

## Two families

### Recursive (`circle_sort_recursive.rs`)

Splits the range at the midpoint and combines three operations: `circle_pass`, `recurse_left`, `recurse_right`. The **`RecursiveOrder`** trait (`orderings.rs`) controls which order these run:

- `PreOrder` — circle pass first, then recurse both halves.
- `LeftMidRight` — left half, circle pass, right half.
- `RightMidLeft` — right half, circle pass, left half.
- `PostOrder` — recurse both halves, then circle pass.

The **shaker recursive** variant (`circle_sort_shaker_recursive.rs`) alternates orderings with recursion depth.

### Bottom-up (`circle_sort_bottom_up.rs`)

Avoids recursion by iterating over all power-of-two circle sizes explicitly. The **`BottomUpDirection`** trait (`directions.rs`) controls traversal order:

- `Decreasing` — largest circles first.
- `Increasing` — smallest circles first.
- `ShakerDecInc` / `ShakerIncDec` — alternating directions.

## Finishing strategies (`finishing.rs`)

Controls how the sort converges after the main circle passes — e.g. repeat until sorted, or switch to a different finisher.

## Files

- `circle_sort_recursive.rs` — recursive family, generic over `RecursiveOrder`.
- `circle_sort_bottom_up.rs` — bottom-up family, generic over `BottomUpDirection`.
- `circle_sort_shaker_recursive.rs` — depth-alternating recursive variant.
- `orderings.rs` — `RecursiveOrder` trait and four implementations.
- `directions.rs` — `BottomUpDirection` trait and four implementations.
- `finishing.rs` — finishing/convergence strategies.
- `sequences.rs` — registration entries (`CIRCLE_ENTRIES` slice).
- `combinations.rs` — generates and registers all variant combinations.

## Status

Compiled but commented out of the active dispatch in `sorts/mod.rs`, pending migration to `sort_family!`.
