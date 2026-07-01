# Component fragments OWNED by this crate (self-describing).

component beap_v_median3
  type     MedianOfThree
  label    median of 3
  provides BeapPiv
  uses     pivot_median3::MedianOfThree
end

component heap_piv_median3
  type     MedianOfThree
  label    median of 3
  provides HeapPiv
  uses     pivot_median3::MedianOfThree
end

component qs_p_median3
  type     MedianOfThree
  label    median of 3
  provides QSPivot QSPivotSingle
  uses     pivot_median3::MedianOfThree
end

component pkp_median3
  type     MedianOfThree
  label    median of 3
  provides PartPivot
  uses     pivot_median3::MedianOfThree
end

