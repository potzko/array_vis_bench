# Component fragments OWNED by this crate (self-describing).

component ss_network16
  type     Network16SmallSort
  label    network: 16
  provides SmallSort NonTrivialSmallSort FunSmallSort SmallSortKind
  max_input 16
  uses     small_sort_network_16::Network16SmallSort
end
