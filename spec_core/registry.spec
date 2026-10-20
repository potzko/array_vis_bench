# Component catalog mirroring the production shapes, to stress the engine:
# type+const components, bool consts, imports, arity-composed selectors and
# partition composites that nest their pivot.
#
# Grammar:
#   component <name>
#     type     <type template>     Rust type expr with {param} holes
#     label    <label template>
#     provides <role> [role...]     roles this component satisfies
#     uses     <path> [path...]     module paths to import (repeatable)
#     slot     <param> <role>
#     const    <param> [values <v>...]  literal only, or a finite value set
#   end

# QuickSort has ONE partition slot. The pivot nests under the partition composite,
# as in the production `CanonicalQuickSort<{part}, {small_sort}>`.
component quick_sort
  type     QuickSort<{partition}, {small_sort}>
  label    quick[{partition}/{small_sort}]
  provides Sort
  uses     crate::quick_sort_lib::quick_sort::QuickSort
  slot     partition Partition
  slot     small_sort SmallSort
end

# Partition composites, as in production: each nests its pivot under itself and
# bakes the arity into the composite. The single composite takes a `PivotSingle`
# and the dual composite a `PivotDual`, so an arity-illegal pairing is
# unrepresentable. `Partition[pivot = p]` refines on the composite's `pivot` slot.
component pk_single
  type     Pivoted<{scheme}, {pivot}>
  label    single<{scheme}, {pivot}>
  provides Partition
  uses     crate::quick_sort_lib::Pivoted
  slot     scheme PartScheme
  slot     pivot PivotSingle
end

component pk_dual
  type     DualPivoted<{pivot}>
  label    dual<{pivot}>
  provides Partition
  uses     crate::quick_sort_lib::DualPivoted
  slot     pivot PivotDual
end

component LL_partition
  type     LeftLeftPartition
  label    LL
  provides PartScheme
  uses     crate::partition_lomuto::LeftLeftPartition
end

# Pivot selectors. They provide the generic `Pivot` role, for queries that
# enumerate every selector, and an arity role (`PivotSingle`/`PivotDual`) that the
# partition composites slot.
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
  slot     a PivotSingle
  slot     b PivotSingle
end

# Small sorts. `insertion` is a TYPE param (strategy) with its threshold baked
# into the type, so a bare `SmallSort` hole can enumerate it.
component no_small_sort
  type     NoSmallSort
  label    none
  provides SmallSort
  uses     crate::small_sorts::NoSmallSort
end

component insertion
  type     InsertionSmallSort<{strategy}, 16>
  label    ins:{strategy}:16
  provides SmallSort
  uses     crate::small_sort_insertion::InsertionSmallSort
  slot     strategy InsertionStrategy
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
  adaptive true
  uses     crate::merge_sort_lib::top_down::TopDownMergeSort
  slot     small_sort SmallSort
  const    ping_pong values false true
  const    early_exit values false true
end

# ── shell sort: single type slot over a gap sequence ──────────────────────────
component shell_sort
  type     ShellSort<{seq}>
  label    shell[{seq}]
  provides Sort
  uses     crate::shell_sort_lib::shell_sort::ShellSort
  slot     seq GapSequence
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

# ── heap sort: a single CONST generic (the d-ary arity) with a declared set of
#    "neat values". `heap_sort(arity = *)` enumerates the set,
#    `heap_sort(arity = ?2@1)` samples it, `heap_sort(arity = 4)` passes one value,
#    and a shared `let k: Width = 4; heap_sort(arity = k)` threads one number to
#    many uses. A query that omits `arity` is an error. The registry only checks
#    MEMBERSHIP in the set, `arity >= 2` (a relation between numbers) stays
#    rustc's job, the redundant backstop.
component heap_sort
  type     HeapSort<{arity}>
  label    heap[d={arity}]
  provides Sort
  uses     crate::heap_lib::HeapSort
  const    arity values 2 3 4
end

# ── recursive grammar: a sort whose `inner` slot is itself a RecSort, bounded by
#    the per-query depth knob. Self-contained role (RecSort) so it nests into a
#    clean linear chain instead of exploding across every Sort.
component recursive_sort
  type     RecursiveSort<{inner}>
  label    rec[{inner}]
  provides Sort RecSort
  uses     crate::recursive_lib::RecursiveSort
  slot     inner RecSort
end

component base_case
  type     BaseCase
  label    base
  provides RecSort
  uses     crate::recursive_lib::BaseCase
end

# ── non-Sort kinds. Each is a ROOT algorithm of its own kind (not a slot
#    filler). A bare role hole (`let pa: PartitionAlgo = .;`) enumerates every
#    component providing it. ─────

component lomuto_partition
  type     LomutoPartition
  label    partition[lomuto]
  provides PartitionAlgo
  uses     crate::partition_ops::LomutoPartition
end

component hoare_partition
  type     HoarePartition
  label    partition[hoare]
  provides PartitionAlgo
  uses     crate::partition_ops::HoarePartition
end

component two_finger_merge
  type     TwoFingerMerge
  label    merge-op[two-finger]
  provides MergeAlgo
  adaptive true
  uses     crate::merge_ops::TwoFingerMerge
end

component reversal_rotation
  type     ReversalRotation
  label    rotation[reversal]
  provides RotationAlgo
  uses     crate::rotation_ops::ReversalRotation
end

component juggling_rotation
  type     JugglingRotation
  label    rotation[juggling]
  provides RotationAlgo
  uses     crate::rotation_ops::JugglingRotation
end
