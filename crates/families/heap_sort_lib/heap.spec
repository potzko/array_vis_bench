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
end
component heap_arity_ternary
  type     Ternary
  label    ternary
  provides HeapArity
end
component heap_arity_16
  type     Base16
  label    16-ary
  provides HeapArity
end
component heap_arity_256
  type     Base256
  label    256-ary
  provides HeapArity
end

# ════════════════════════ HeapDir (2, SHARED) ════════════════════════
# Only the two ascending-producing directions carry the marker (legacy parity).
component heap_dir_min_reverse
  type     MinReverse
  label    min reverse
  provides HeapDir
end
component heap_dir_max_forward
  type     MaxForward
  label    max forward
  provides HeapDir
end

# ════════════════════════ HeapPart (3, SHARED) ════════════════════════
component heap_part_left_left
  type     HeapLeftLeft
  label    left-left pointer
  provides HeapPart
end
component heap_part_left_right
  type     HeapLeftRight
  label    left-right pointer
  provides HeapPart
end
component heap_part_block
  type     HeapBlock
  label    block
  provides HeapPart
end

# ════════════════════════ HeapPiv (5, heap-only) ════════════════════════

# ════════════════════════ DeepHeapifyClassic (2) ════════════════════════
component heap_dh_iterative
  type     Iterative
  label    iterative
  provides DeepHeapifyClassic BeapDeepHeapifyClassic RecDeepHeapify
end
component heap_dh_recursive
  type     Recursive
  label    recursive
  provides DeepHeapifyClassic RecDeepHeapify
end

# ════════════════ DeepHeapifyQuick (3 composites, each <HP, V>) ════════════════
component heap_qdh_sequential
  type     SequentialQuickDeepHeapify<{hp}, {v}>
  label    sequential<part: {hp}, pivot: {v}>
  provides DeepHeapifyQuick
  slot     hp HeapPart
  slot     v  HeapPiv
end
component heap_qdh_recursive_partial
  type     RecursivePartialQuickDeepHeapify<{hp}, {v}>
  label    recursive partition<part: {hp}, pivot: {v}>
  provides DeepHeapifyQuick
  slot     hp HeapPart
  slot     v  HeapPiv
end
component heap_qdh_stack_partial
  type     StackPartialQuickDeepHeapify<{hp}, {v}>
  label    stack partition<part: {hp}, pivot: {v}>
  provides DeepHeapifyQuick
  slot     hp HeapPart
  slot     v  HeapPiv
end

# ════════════════ DeepHeapifyDual (3 dual-pivot selectors) ════════════════
component heap_dps_first
  type     StackDualPivotPartialQuickDeepHeapify<CombinedSelector<FirstElement, FirstElement>>
  label    stack dual-pivot partition<first>
  provides DeepHeapifyDual
end
component heap_dps_middle
  type     StackDualPivotPartialQuickDeepHeapify<CombinedSelector<MiddleElement, MiddleElement>>
  label    stack dual-pivot partition<middle>
  provides DeepHeapifyDual
end
component heap_dps_ninther
  type     StackDualPivotPartialQuickDeepHeapify<NintherDualPivot>
  label    stack dual-pivot partition<ninther 1/3 + 2/3>
  provides DeepHeapifyDual
end

# ════════════════════════ DRIVERS (3 unique heads) ════════════════════════
component heap_sort_classic
  type     HeapSortClassicOf<{a}, {d}, {dh}>
  label    heap sort<arity: {a}, dir: {d}, build: {dh}>
  provides Sort
  menu     heap sorts / d-ary / classic
  slot     a  HeapArity
  slot     d  HeapDir
  slot     dh DeepHeapifyClassic
end
component heap_sort_quick_build
  type     HeapSortQuickBuildOf<{a}, {d}, {dh}>
  label    heap sort quick build<arity: {a}, dir: {d}, build: {dh}>
  provides Sort
  menu     heap sorts / d-ary / quick build / single pivot
  slot     a  HeapArity
  slot     d  HeapDir
  slot     dh DeepHeapifyQuick
end
component heap_sort_dual_build
  type     HeapSortDualBuildOf<{a}, {d}, {dh}>
  label    heap sort quick build dual pivot<arity: {a}, dir: {d}, build: {dh}>
  provides Sort
  menu     heap sorts / d-ary / quick build / dual pivot
  slot     a  HeapArity
  slot     d  HeapDir
  slot     dh DeepHeapifyDual
end
component weak_heap_sort
  type     HeapSort<WeakHeapSort<{d}, {r}>>
  label    weak heap sort<dir: {d}, storage: {r}>
  provides Sort
  menu     heap sorts / weak
  # Listed rather than derived: WeakHeapSort lives in weak_heap_sort_lib, which depends
  # on this crate, so the owning crate cannot reach it.
  uses     heap_sort_lib::HeapSort weak_heap_sort_lib::WeakHeapSort
  slot     d HeapDir
  slot     r ReverseStorage
end

component beap_qdh_sequential
  type     SequentialQuickDeepHeapify<{hp}, {v}>
  label    sequential<part: {hp}, pivot: {v}>
  provides BeapDeepHeapifyQuick
  slot     hp HeapPart
  slot     v  BeapPiv
end

component beap_qdh_recursive_partial
  type     RecursivePartialQuickDeepHeapify<{hp}, {v}>
  label    recursive partition<part: {hp}, pivot: {v}>
  provides BeapDeepHeapifyQuick
  slot     hp HeapPart
  slot     v  BeapPiv
end

component beap_qdh_stack_partial
  type     StackPartialQuickDeepHeapify<{hp}, {v}>
  label    stack partition<part: {hp}, pivot: {v}>
  provides BeapDeepHeapifyQuick
  slot     hp HeapPart
  slot     v  BeapPiv
end

component beap_dpqdh_stack_dual
  type     StackDualPivotPartialQuickDeepHeapify<{dps}>
  label    stack dual-pivot partition<dps: {dps}>
  provides BeapDeepHeapifyDual
  slot     dps BeapDualPivot
end
