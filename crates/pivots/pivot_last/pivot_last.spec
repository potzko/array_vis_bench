# Component fragments OWNED by this crate (self-describing).

component qs_p_last
  type     LastElement
  label    last
  provides QSPivot QSPivotSingle
  uses     pivot_last::LastElement
end

component pkp_last
  type     LastElement
  label    last
  provides PartPivot
  uses     pivot_last::LastElement
end

