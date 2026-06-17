# Cycle-sort family catalog fragment — OWNED by this crate, gathered by
# `spec_catalog` via `[package.metadata.array_vis_bench] spec = "cycle.spec"`.
# Zero-axis: a single concrete sort, no slots.

component cycle_sort
  type     CycleSort
  label    cycle sort
  provides Sort
  category Sort
  menu     cycle sorts
  uses     cycle_sort_lib::CycleSort
end
