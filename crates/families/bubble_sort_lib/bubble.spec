# Bubble-sort family catalog fragment — OWNED by this crate, gathered by
# `spec_catalog` via `[package.metadata.array_vis_bench] spec = "bubble.spec"`.
#
# Three zero-axis base sorts + the single-axis `OddEvenBubbleSort<{small}>` over
# the NonTrivialSmallSort role. The small-sort TYPES live in sibling leaf crates
# (already `spec_catalog` deps); `uses` paths resolve in the consumer.

# ── Base sorts (zero-axis) ────────────────────────────────────────────────────
component bubble_sort
  type     BubbleSort
  label    bubble sort
  provides Sort
  category Sort
  menu     bubble sorts
  uses     bubble_sort_lib::BubbleSort
end

component shaker_sort
  type     ShakerSort
  label    shaker sort
  provides Sort
  category Sort
  menu     bubble sorts
  uses     bubble_sort_lib::ShakerSort
end

component bubble_sort_recursive
  type     BubbleSortRecursive
  label    bubble sort recursive
  provides Sort
  category Sort
  menu     bubble sorts
  uses     bubble_sort_lib::BubbleSortRecursive
end

# ── Odd-even bubble sort (single-axis over a non-trivial small sort) ──────────
component odd_even_bubble_sort
  type     OddEvenBubbleSort<{small}>
  label    odd-even bubble sort<small: {small}>
  provides Sort
  category Sort
  menu     bubble sorts / odd-even bubble sort
  uses     bubble_sort_lib::OddEvenBubbleSort
  slot     small NonTrivialSmallSort nt_insertion32
end

# NonTrivialSmallSort components (same types quick uses as SmallSort, declared
# here under the NonTrivialSmallSort role for odd-even's cutoff slot).
component nt_insertion32
  type     InsertionSmallSort<LinearInsertion, 32>
  label    insertion: 32
  provides NonTrivialSmallSort
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component nt_network16
  type     Network16SmallSort
  label    network: 16
  provides NonTrivialSmallSort
  uses     small_sort_network_16::Network16SmallSort
end

component nt_size2
  type     Size2SmallSort
  label    size: 2
  provides NonTrivialSmallSort
  uses     small_sort_basic::Size2SmallSort
end
