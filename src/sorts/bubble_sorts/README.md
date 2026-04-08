# bubble_sorts

Bubble sort and its close relatives — O(N^2) comparison sorts that work by repeatedly swapping adjacent out-of-order elements.

## How bubble sort works

Repeatedly sweep through the array, comparing each adjacent pair and swapping if they're out of order. After each full pass, the largest unseen element "bubbles" to its final position. Repeat until a pass makes no swaps.

Simple and stable, but O(N^2) — mainly useful as a baseline and for visualisation (the sweeping pattern is visually distinctive).

## Variants

- `bubble_sort.rs` — **Bubble sort**: the standard version. Forward passes until no swaps.
- `bubble_sort_recursive.rs` — **Bubble sort recursive**: each "pass" is a recursive call that sorts `arr[..n-1]` after bubbling the max to position `n-1`. Same comparisons, different control flow.
- `odd_even_bubble_sort.rs` — **Odd-even bubble sort** (brick sort): alternates between comparing odd-indexed pairs (1-2, 3-4, ...) and even-indexed pairs (0-1, 2-3, ...). All comparisons in one phase are independent, making this parallelisable (though this implementation is sequential).
- `shaker_sort.rs` — **Shaker sort** (cocktail sort): bidirectional bubble sort — alternates forward and backward passes. Handles "turtles" (small elements near the end) better than unidirectional bubble sort.

## Status

Compiled but commented out of the active dispatch in `sorts/mod.rs`, pending migration to `sort_family!`.
