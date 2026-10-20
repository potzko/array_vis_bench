# Component fragments OWNED by this crate (self-describing).

component piv_last
  type     LastElement
  label    last
  provides PartPivot
  uses     pivot_last::LastElement
end
