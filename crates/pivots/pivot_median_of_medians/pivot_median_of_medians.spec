# Component fragments OWNED by this crate (self-describing).

component beap_v_median_of_medians
  type     MedianOfMedians
  label    median of medians
  provides BeapPiv
  uses     pivot_median_of_medians::MedianOfMedians
end

component p_median_of_medians
  type     MedianOfMedians
  label    median of medians
  provides Pivot PivotSingle
  uses     pivot_median_of_medians::MedianOfMedians
end

component heap_piv_median_of_medians
  type     MedianOfMedians
  label    median of medians
  provides HeapPiv
  uses     pivot_median_of_medians::MedianOfMedians
end

component pkp_median_med
  type     MedianOfMedians
  label    median of medians
  provides PartPivot
  uses     pivot_median_of_medians::MedianOfMedians
end

