# Heap-sort family catalog fragment — OWNED by heap_sort_lib, gathered by
# `spec_catalog` via [package.metadata.array_vis_bench] spec = "heap.spec".
#
# Three driver heads (classic d-ary / quickselect-build single-pivot / quickselect-
# build dual-pivot) over the shared Arity x Direction facets. The drivers wrap the
# real HeapSort<NaryHeapSort<ArityHeap<A,D>, DH>> chain with unique type-heads (the
# legacy types all share the `HeapSort` head, which AVBS resolves first-wins).
#
# This fragment OWNS the heap-cluster's shared role components HeapDir (2) and
# HeapPart (3) — weak_heap and beap REFERENCE these roles (do not redeclare).
# Everything else (HeapArity, HeapPiv, DeepHeapify*) is heap-specific.

# ════════════════════════ Arity (4) ════════════════════════
component heap_arity_binary
  type     Binary
  label    binary
  provides HeapArity
  uses     heap_sort_lib::Binary
end
component heap_arity_ternary
  type     Ternary
  label    ternary
  provides HeapArity
  uses     heap_sort_lib::Ternary
end
component heap_arity_16
  type     Base16
  label    16-ary
  provides HeapArity
  uses     heap_sort_lib::Base16
end
component heap_arity_256
  type     Base256
  label    256-ary
  provides HeapArity
  uses     heap_sort_lib::Base256
end

# ════════════════════════ HeapDir (2, SHARED) ════════════════════════
# Only the two ascending-producing directions carry the marker (legacy parity).
component heap_dir_min_reverse
  type     MinReverse
  label    min reverse
  provides HeapDir
  uses     heap_sort_lib::MinReverse
end
component heap_dir_max_forward
  type     MaxForward
  label    max forward
  provides HeapDir
  uses     heap_sort_lib::MaxForward
end

# ════════════════════════ HeapPart (3, SHARED) ════════════════════════
component heap_part_left_left
  type     HeapLeftLeft
  label    left-left pointer
  provides HeapPart
  uses     heap_sort_lib::HeapLeftLeft
end
component heap_part_left_right
  type     HeapLeftRight
  label    left-right pointer
  provides HeapPart
  uses     heap_sort_lib::HeapLeftRight
end
component heap_part_block
  type     HeapBlock
  label    block
  provides HeapPart
  uses     heap_sort_lib::HeapBlock
end

# ════════════════════════ HeapPiv (5, heap-only) ════════════════════════

# ════════════════════════ DeepHeapifyClassic (2) ════════════════════════
component heap_dh_iterative
  type     Iterative
  label    iterative
  provides DeepHeapifyClassic BeapDeepHeapifyClassic RecDeepHeapify
  uses     heap_sort_lib::Iterative
end
component heap_dh_recursive
  type     Recursive
  label    recursive
  provides DeepHeapifyClassic RecDeepHeapify
  uses     heap_sort_lib::Recursive
end

# ════════════════ DeepHeapifyQuick (3 composites, each <HP, V>) ════════════════
component heap_qdh_sequential
  type     SequentialQuickDeepHeapify<{hp}, {v}>
  label    sequential<part: {hp}, pivot: {v}>
  provides DeepHeapifyQuick
  uses     heap_sort_lib::SequentialQuickDeepHeapify
  slot     hp HeapPart
  slot     v  HeapPiv
end
component heap_qdh_recursive_partial
  type     RecursivePartialQuickDeepHeapify<{hp}, {v}>
  label    recursive partition<part: {hp}, pivot: {v}>
  provides DeepHeapifyQuick
  uses     heap_sort_lib::RecursivePartialQuickDeepHeapify
  slot     hp HeapPart
  slot     v  HeapPiv
end
component heap_qdh_stack_partial
  type     StackPartialQuickDeepHeapify<{hp}, {v}>
  label    stack partition<part: {hp}, pivot: {v}>
  provides DeepHeapifyQuick
  uses     heap_sort_lib::StackPartialQuickDeepHeapify
  slot     hp HeapPart
  slot     v  HeapPiv
end

# ════════════════ DeepHeapifyDual (3 dual-pivot selectors) ════════════════
component heap_dps_first
  type     StackDualPivotPartialQuickDeepHeapify<CombinedSelector<FirstElement, FirstElement>>
  label    stack dual-pivot partition<first>
  provides DeepHeapifyDual
  uses     heap_sort_lib::StackDualPivotPartialQuickDeepHeapify quick_sort_lib::CombinedSelector pivot_first::FirstElement
end
component heap_dps_middle
  type     StackDualPivotPartialQuickDeepHeapify<CombinedSelector<MiddleElement, MiddleElement>>
  label    stack dual-pivot partition<middle>
  provides DeepHeapifyDual
  uses     heap_sort_lib::StackDualPivotPartialQuickDeepHeapify quick_sort_lib::CombinedSelector pivot_middle::MiddleElement
end
component heap_dps_ninther
  type     StackDualPivotPartialQuickDeepHeapify<NintherDualPivot>
  label    stack dual-pivot partition<ninther 1/3 + 2/3>
  provides DeepHeapifyDual
  uses     heap_sort_lib::StackDualPivotPartialQuickDeepHeapify quick_sort_lib::NintherDualPivot
end

# ════════════════════════ DRIVERS (3 unique heads) ════════════════════════
component heap_sort_classic
  type     HeapSortClassicOf<{a}, {d}, {dh}>
  label    heap sort<arity: {a}, dir: {d}, build: {dh}>
  provides Sort
  menu     heap sorts / d-ary / classic
  uses     heap_sort_lib::HeapSortClassicOf
  slot     a  HeapArity
  slot     d  HeapDir
  slot     dh DeepHeapifyClassic
end
component heap_sort_quick_build
  type     HeapSortQuickBuildOf<{a}, {d}, {dh}>
  label    heap sort quick build<arity: {a}, dir: {d}, build: {dh}>
  provides Sort
  menu     heap sorts / d-ary / quick build / single pivot
  uses     heap_sort_lib::HeapSortQuickBuildOf
  slot     a  HeapArity
  slot     d  HeapDir
  slot     dh DeepHeapifyQuick
end
component heap_sort_dual_build
  type     HeapSortDualBuildOf<{a}, {d}, {dh}>
  label    heap sort quick build dual pivot<arity: {a}, dir: {d}, build: {dh}>
  provides Sort
  menu     heap sorts / d-ary / quick build / dual pivot
  uses     heap_sort_lib::HeapSortDualBuildOf
  slot     a  HeapArity
  slot     d  HeapDir
  slot     dh DeepHeapifyDual
end
component weak_heap_sort
  type     HeapSort<WeakHeapSort<{d}, {r}>>
  label    weak heap sort<dir: {d}, storage: {r}>
  provides Sort
  menu     heap sorts / weak
  uses     heap_sort_lib::HeapSort weak_heap_sort_lib::WeakHeapSort
  slot     d HeapDir
  slot     r ReverseStorage
end

component beap_qdh_sequential
  type     SequentialQuickDeepHeapify<{hp}, {v}>
  label    sequential<part: {hp}, pivot: {v}>
  provides BeapDeepHeapifyQuick
  uses     heap_sort_lib::SequentialQuickDeepHeapify
  slot     hp HeapPart
  slot     v  BeapPiv
end

component beap_qdh_recursive_partial
  type     RecursivePartialQuickDeepHeapify<{hp}, {v}>
  label    recursive partition<part: {hp}, pivot: {v}>
  provides BeapDeepHeapifyQuick
  uses     heap_sort_lib::RecursivePartialQuickDeepHeapify
  slot     hp HeapPart
  slot     v  BeapPiv
end

component beap_qdh_stack_partial
  type     StackPartialQuickDeepHeapify<{hp}, {v}>
  label    stack partition<part: {hp}, pivot: {v}>
  provides BeapDeepHeapifyQuick
  uses     heap_sort_lib::StackPartialQuickDeepHeapify
  slot     hp HeapPart
  slot     v  BeapPiv
end

component beap_dpqdh_stack_dual
  type     StackDualPivotPartialQuickDeepHeapify<{dps}>
  label    stack dual-pivot partition<dps: {dps}>
  provides BeapDeepHeapifyDual
  uses     heap_sort_lib::StackDualPivotPartialQuickDeepHeapify
  slot     dps BeapDualPivot
end
