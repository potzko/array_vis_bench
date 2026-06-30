# Component fragments OWNED by this crate (self-describing).

component ins_linear
  type     LinearInsertion
  label    linear
  provides InsertionStrategy
  uses     small_sort_insertion_strategy::LinearInsertion
end

component ins_binary
  type     BinaryInsertion
  label    binary
  provides InsertionStrategy
  uses     small_sort_insertion_strategy::BinaryInsertion
end

