# Component fragments OWNED by this crate (self-describing).

component beap_v_first
  type     FirstElement
  label    first
  provides BeapPiv
  uses     pivot_first::FirstElement
end

component p_first
  type     FirstElement
  label    first
  provides Pivot PivotSingle
  uses     pivot_first::FirstElement
end

component heap_piv_first
  type     FirstElement
  label    first
  provides HeapPiv
  uses     pivot_first::FirstElement
end

component qs_p_first
  type     FirstElement
  label    first
  provides QSPivot QSPivotSingle
  uses     pivot_first::FirstElement
end

component pkp_first
  type     FirstElement
  label    first
  provides PartPivot
  uses     pivot_first::FirstElement
end

