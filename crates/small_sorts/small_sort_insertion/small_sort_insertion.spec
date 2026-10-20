# Component fragments OWNED by this crate (self-describing).

component ss_insertion32
  type     InsertionSmallSort<LinearInsertion, 32>
  label    insertion: 32
  provides SmallSort NonTrivialSmallSort FunSmallSort SmallSortKind
  category SmallSort
  max_input 32
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component ss_insertion16
  type     InsertionSmallSort<LinearInsertion, 16>
  label    insertion: 16
  provides SmallSortKind
  category SmallSort
  max_input 16
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component ss_binary_insertion16
  type     InsertionSmallSort<BinaryInsertion, 16>
  label    binary insertion: 16
  provides SmallSortKind
  category SmallSort
  max_input 16
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::BinaryInsertion
end

component ss_binary_insertion32
  type     InsertionSmallSort<BinaryInsertion, 32>
  label    binary insertion: 32
  provides SmallSortKind
  category SmallSort
  max_input 32
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::BinaryInsertion
end
