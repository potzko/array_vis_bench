# Component fragments OWNED by this crate (self-describing).
#
# `pkp_middle` (the shared `PartPivot` atom) is declared FIRST so it wins the
# `MiddleElement` type-head on collision (see pivot_first.spec for the rationale) —
# an explicit `MiddleElement` reference in a query resolves to the PartPivot
# component. beap/heap reach their selectors via role, not head.

component pkp_middle
  type     MiddleElement
  label    middle
  provides PartPivot
  uses     pivot_middle::MiddleElement
end

component beap_v_middle
  type     MiddleElement
  label    middle
  provides BeapPiv
  uses     pivot_middle::MiddleElement
end

component heap_piv_middle
  type     MiddleElement
  label    middle
  provides HeapPiv
  uses     pivot_middle::MiddleElement
end
