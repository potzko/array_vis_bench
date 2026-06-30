# Component fragments OWNED by this crate (self-describing).

component nt_insertion32
  type     InsertionSmallSort<LinearInsertion, 32>
  label    insertion: 32
  provides NonTrivialSmallSort
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component fs_ss_insertion32
  type     InsertionSmallSort<LinearInsertion, 32>
  label    insertion: 32
  provides FunSmallSort
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component ss_insertion32
  type     InsertionSmallSort<LinearInsertion, 32>
  label    insertion: 32
  provides SmallSort
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component sk_ins_lin16
  type     InsertionSmallSort<LinearInsertion, 16>
  label    insertion: 16
  provides SmallSortKind
  category SmallSort
  max_input 16
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component sk_ins_lin32
  type     InsertionSmallSort<LinearInsertion, 32>
  label    insertion: 32
  provides SmallSortKind
  category SmallSort
  max_input 32
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::LinearInsertion
end

component sk_ins_bin16
  type     InsertionSmallSort<BinaryInsertion, 16>
  label    binary insertion: 16
  provides SmallSortKind
  category SmallSort
  max_input 16
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::BinaryInsertion
end

component sk_ins_bin32
  type     InsertionSmallSort<BinaryInsertion, 32>
  label    binary insertion: 32
  provides SmallSortKind
  category SmallSort
  max_input 32
  uses     small_sort_insertion::InsertionSmallSort small_sort_insertion_strategy::BinaryInsertion
end

