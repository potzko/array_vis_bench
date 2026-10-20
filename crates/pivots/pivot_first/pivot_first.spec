# Component fragments OWNED by this crate (self-describing).

component piv_first
  type     FirstElement
  label    first
  provides PartPivot HeapPiv BeapPiv
  uses     pivot_first::FirstElement
end
