# Component catalog — now mirroring the REAL sort type shapes (see survey), to
# stress the engine: type+const components, bool consts, imports, arity-composed
# selectors, and the sibling-arity QuickSort layout.
#
# Grammar:
#   component <name>
#     type     <type template>     Rust type expr with {param} holes
#     label    <label template>
#     provides <role> [role...]     roles this component satisfies
#     uses     <path> [path...]     module paths to import (repeatable)
#     slot     <param> <role> [<default-component>]
#     const    <param> [<default>]  default may be an int or true/false
#   end

# ── QuickSort — FLAT: partition and pivot are SIBLING slots, matching the real
#    `QuickSort<P: PartitionScheme, V: PivotInput, SS: SmallSort>`.
component quick_sort
  type     QuickSort<{partition}, {pivot}, {small_sort}>
  label    quick[{partition}/{pivot}/{small_sort}]
  provides Sort
  uses     crate::quick_sort_lib::quick_sort::QuickSort
  slot     partition Partition
  slot     pivot Pivot
  slot     small_sort SmallSort no_small_sort
end

# ── partitions (real ones are unit structs; pivot is NOT nested here) ─────────
component LL_partition
  type     LeftLeftPartition
  label    LL
  provides Partition
  uses     crate::partition_lomuto::LeftLeftPartition
end

component dual_pivot_partition
  type     DualPivotPartition
  label    dual
  provides Partition
  uses     crate::quick_sort_lib::yaroslavskiy::DualPivotPartition
end

# ── pivot selectors. They provide both the generic `Pivot` role (so the flat
#    quick_sort slot accepts them) and an arity role (`PivotSingle`/`PivotDual`).
component first_element
  type     FirstElement
  label    first
  provides Pivot PivotSingle
  uses     crate::pivots::FirstElement
end

component middle_element
  type     MiddleElement
  label    mid
  provides Pivot PivotSingle
  uses     crate::pivots::MiddleElement
end

component ninther_dual
  type     NintherDualPivot
  label    ninther
  provides Pivot PivotDual
  uses     crate::quick_sort_lib::pivot_selectors::NintherDualPivot
end

# arity-composed: a dual selector built from two single selectors
component combined
  type     CombinedSelector<{a}, {b}>
  label    combined<{a},{b}>
  provides Pivot PivotDual
  uses     crate::quick_sort_lib::pivot_selectors::CombinedSelector
  slot     a PivotSingle first_element
  slot     b PivotSingle middle_element
end

# ── small sorts. `insertion` is a TYPE param (strategy) + a CONST (threshold). ─
component no_small_sort
  type     NoSmallSort
  label    none
  provides SmallSort
  uses     crate::small_sorts::NoSmallSort
end

component insertion
  type     InsertionSmallSort<{strategy}, {N}>
  label    ins:{N}
  provides SmallSort
  uses     crate::small_sort_insertion::InsertionSmallSort
  slot     strategy InsertionStrategy linear
  const    N 16
end

component linear
  type     LinearInsertion
  label    lin
  provides InsertionStrategy
  uses     crate::small_sort_insertion_strategy::LinearInsertion
end

component binary
  type     BinaryInsertion
  label    bin
  provides InsertionStrategy
  uses     crate::small_sort_insertion_strategy::BinaryInsertion
end

# ── merge sort: TYPE param + TWO bool consts ──────────────────────────────────
component top_down_merge
  type     TopDownMergeSort<{small_sort}, {ping_pong}, {early_exit}>
  label    merge[{small_sort}/pp={ping_pong}/ee={early_exit}]
  provides Sort
  uses     crate::merge_sort_lib::top_down::TopDownMergeSort
  slot     small_sort SmallSort no_small_sort
  const    ping_pong false
  const    early_exit false
end

# ── shell sort: single type slot over a gap sequence ──────────────────────────
component shell_sort
  type     ShellSort<{seq}>
  label    shell[{seq}]
  provides Sort
  uses     crate::shell_sort_lib::shell_sort::ShellSort
  slot     seq GapSequence knuth
end

component classic
  type     Classic
  label    classic
  provides GapSequence
  uses     crate::shell_sort_lib::sequences::Classic
end

component knuth
  type     Knuth
  label    knuth
  provides GapSequence
  uses     crate::shell_sort_lib::sequences::Knuth
end

component ciura
  type     Ciura
  label    ciura
  provides GapSequence
  uses     crate::shell_sort_lib::sequences::Ciura
end
