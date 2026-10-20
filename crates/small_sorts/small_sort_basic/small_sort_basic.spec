# Component fragments OWNED by this crate (self-describing).

component ss_size1
  type     Size1SmallSort
  label    size: 1
  provides SmallSort SmallSortKind
  category SmallSort
  max_input 1
  uses     small_sort_basic::Size1SmallSort
end

component ss_size2
  type     Size2SmallSort
  label    size: 2
  provides SmallSort NonTrivialSmallSort SmallSortKind
  category SmallSort
  max_input 2
  uses     small_sort_basic::Size2SmallSort
end
