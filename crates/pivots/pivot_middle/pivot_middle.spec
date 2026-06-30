# Component fragments OWNED by this crate (self-describing).

component beap_v_middle
  type     MiddleElement
  label    middle
  provides BeapPiv
  uses     pivot_middle::MiddleElement
end

component p_middle
  type     MiddleElement
  label    middle
  provides Pivot PivotSingle
  uses     pivot_middle::MiddleElement
end

component heap_piv_middle
  type     MiddleElement
  label    middle
  provides HeapPiv
  uses     pivot_middle::MiddleElement
end

component qs_p_middle
  type     MiddleElement
  label    middle
  provides QSPivot QSPivotSingle
  uses     pivot_middle::MiddleElement
end

component pkp_middle
  type     MiddleElement
  label    middle
  provides PartPivot
  uses     pivot_middle::MiddleElement
end

