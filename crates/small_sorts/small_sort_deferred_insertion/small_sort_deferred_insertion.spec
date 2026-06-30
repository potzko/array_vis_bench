# Component fragments OWNED by this crate (self-describing).

component dss_lin16
  type     DeferredInsertion<LinearInsertion, 16>
  label    deferred insertion: 16
  provides DeferredSmallSort
  uses     small_sort_deferred_insertion::DeferredInsertion small_sort_insertion_strategy::LinearInsertion
end

component dss_lin32
  type     DeferredInsertion<LinearInsertion, 32>
  label    deferred insertion: 32
  provides DeferredSmallSort
  uses     small_sort_deferred_insertion::DeferredInsertion small_sort_insertion_strategy::LinearInsertion
end

component dss_bin16
  type     DeferredInsertion<BinaryInsertion, 16>
  label    deferred binary insertion: 16
  provides DeferredSmallSort
  uses     small_sort_deferred_insertion::DeferredInsertion small_sort_insertion_strategy::BinaryInsertion
end

component dss_bin32
  type     DeferredInsertion<BinaryInsertion, 32>
  label    deferred binary insertion: 32
  provides DeferredSmallSort
  uses     small_sort_deferred_insertion::DeferredInsertion small_sort_insertion_strategy::BinaryInsertion
end

