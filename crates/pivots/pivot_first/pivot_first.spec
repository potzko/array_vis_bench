# Component fragments OWNED by this crate (self-describing).
#
# `pkp_first` (the shared `PartPivot` atom) is declared FIRST so it wins the
# `FirstElement` type-head on collision — the AVBS frontend resolves an explicit
# `FirstElement` reference (e.g. inside `CombinedSelector<FirstElement, …>`) to the
# PartPivot component, which is the only one whose role satisfies a partition
# pivot slot. beap/heap reach their own selectors via the BeapPiv/HeapPiv ROLE
# (never by head), so their resolution is unaffected by this ordering.

component pkp_first
  type     FirstElement
  label    first
  provides PartPivot
  uses     pivot_first::FirstElement
end

component beap_v_first
  type     FirstElement
  label    first
  provides BeapPiv
  uses     pivot_first::FirstElement
end

component heap_piv_first
  type     FirstElement
  label    first
  provides HeapPiv
  uses     pivot_first::FirstElement
end
