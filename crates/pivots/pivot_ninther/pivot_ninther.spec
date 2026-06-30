# Component fragments OWNED by this crate (self-describing).

component p_ninther
  type     Ninther
  label    ninther
  provides Pivot PivotSingle
  uses     pivot_ninther::Ninther
end

component heap_piv_ninther
  type     Ninther
  label    ninther
  provides HeapPiv
  uses     pivot_ninther::Ninther
end

component qs_p_ninther
  type     Ninther
  label    ninther
  provides QSPivot QSPivotSingle
  uses     pivot_ninther::Ninther
end

component pkp_ninther
  type     Ninther
  label    ninther
  provides PartPivot
  uses     pivot_ninther::Ninther
end

