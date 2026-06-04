# Component catalog — the "registry of texts describing algorithms and types".
# This is the shared SPECIFICATION contract. Both front-ends (the inline macro
# and the enumerating generator) read this exact file; neither hardcodes types.
#
# Grammar (line-oriented):
#   component <dsl-name>          start a block
#     type     <type template>   Rust type expr with {param} holes; verbatim, may contain spaces/commas
#     label    <label template>  human label with {param} holes
#     provides <role> [role...]   roles this component can satisfy (its "output type")
#     slot     <param> <role> [<default-component>]   a nested type slot requiring <role>
#     const    <param> [<default-int>]                a positional const-generic
#   end
#
# Legality is expressed by ROLES, not by a recursion heuristic: a slot accepts
# only components that `provides` its role. rustc remains the ultimate backstop.

component quick_sort
  type     QuickSort<{partition}, {small_sort}>
  label    quick[{partition}/{small_sort}]
  provides Sort
  slot     partition Partition
  slot     small_sort SmallSort no_small_sort
end

# ── partitions: each OWNS its pivot selector (nested), and the slot's role
#    encodes pivot arity — a single-pivot partition cannot accept a dual selector.
component LL_partition
  type     LeftLeftPartition<{pivot}>
  label    LL<{pivot}>
  provides Partition
  slot     pivot SinglePivot first_element
end

component LR_partition
  type     HoarePartition<{pivot}>
  label    LR<{pivot}>
  provides Partition
  slot     pivot SinglePivot first_element
end

component dual_pivot
  type     DualPivotPartition<{pivot}>
  label    dual<{pivot}>
  provides Partition
  slot     pivot DualPivot tukey_dual
end

# ── pivot selectors ───────────────────────────────────────────────────────────
component first_element
  type     FirstElement
  label    first
  provides SinglePivot
end

component middle_element
  type     MiddleElement
  label    mid
  provides SinglePivot
end

component median3
  type     MedianOfThree
  label    med3
  provides SinglePivot
end

component tukey_dual
  type     TukeyNinther
  label    tukey
  provides DualPivot
end

# ── small sorts ─────────────────────────────────────────────────────────────
component no_small_sort
  type     NoSmallSort
  label    none
  provides SmallSort
end

component insertion_sort
  type     InsertionSmallSort<{N}>
  label    ins:{N}
  provides SmallSort
  const    N 16
end
