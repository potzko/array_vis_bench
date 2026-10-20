# Insertion-sort family catalog fragment — OWNED by this crate.
#
# Gathered by `spec_catalog` via `[package.metadata.array_vis_bench] spec =
# "insertion.spec"`. Declares the `InsertionSort<{strategy}>` driver and the two
# strategies (linear / binary). The strategy TYPES live in
# `small_sort_insertion_strategy` (a dependency of this crate); the `uses` paths
# resolve in the consumer (`spec_catalog`).

component insertion_sort
  type     InsertionSort<{strategy}>
  label    insertion sort<strategy: {strategy}>
  provides Sort
  menu     insertion sorts
  uses     insertion_sort_lib::InsertionSort
  slot     strategy InsertionStrategy
end
