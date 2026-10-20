# Beap-sort family catalog fragment — OWNED by beap_sort_lib.
# Gathered via [package.metadata.array_vis_bench] spec = "beap.spec".
#
# Three driver heads (classic / quick-build single-pivot / quick-build dual-pivot)
# wrapping HeapSort<NaryHeapSort<BeapHeap<D>, DH>> with distinct type-heads. Beap
# REUSES heap.spec's shared roles HeapDir (2) + HeapPart (3) but has its OWN pivot
# axis (BeapPiv = 4, no ninther) and its own deep-heapify roles (beap-classic has
# only Iterative, vs heap-classic's 2), so those are beap-prefixed.
#
# Per-variant complexity is O(N^1.5) (N_SQRT_N) — the √N beap heapify — distinct
# from d-ary heap's N log N. Total: 2 + 72 + 6 = 80.

# ════════════════════════ BeapPiv (4, beap-only) ════════════════════════

# ════════════════════════ DeepHeapify — classic (1, beap-only) ════════════════════════

# ════════════════ DeepHeapify — quick single-pivot (3 composites, hp=HeapPart shared, v=BeapPiv) ════════════════

# ════════════════ DeepHeapify — dual-pivot (1 composite over a DPS selector) ════════════════

# ════════════════════════ DRIVERS (3 distinct heads) ════════════════════════
component beap_sort_classic
  type     BeapSortClassicOf<{d}, {dh}>
  label    beap sort<dir: {d}, build: {dh}>
  provides Sort
  menu     heap sorts / beap / classic
  uses     beap_sort_lib::spec_drivers::BeapSortClassicOf
  slot     d  HeapDir
  slot     dh BeapDeepHeapifyClassic
  max_input 10000
end
component beap_sort_quick_build
  type     BeapSortQuickOf<{d}, {dh}>
  label    beap sort quick build<dir: {d}, build: {dh}>
  provides Sort
  menu     heap sorts / beap / quick build / single pivot
  uses     beap_sort_lib::spec_drivers::BeapSortQuickOf
  slot     d  HeapDir
  slot     dh BeapDeepHeapifyQuick
  max_input 10000
end
component beap_sort_dual_build
  type     BeapSortDualOf<{d}, {dh}>
  label    beap sort quick build dual pivot<dir: {d}, build: {dh}>
  provides Sort
  menu     heap sorts / beap / quick build / dual pivot
  uses     beap_sort_lib::spec_drivers::BeapSortDualOf
  slot     d  HeapDir
  slot     dh BeapDeepHeapifyDual
  max_input 10000
end
