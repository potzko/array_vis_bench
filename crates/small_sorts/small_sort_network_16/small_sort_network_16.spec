# Component fragments OWNED by this crate (self-describing).

component nt_network16
  type     Network16SmallSort
  label    network: 16
  provides NonTrivialSmallSort
  uses     small_sort_network_16::Network16SmallSort
end

component fs_ss_network16
  type     Network16SmallSort
  label    network: 16
  provides FunSmallSort
  uses     small_sort_network_16::Network16SmallSort
end

component ss_network16
  type     Network16SmallSort
  label    network: 16
  provides SmallSort
  uses     small_sort_network_16::Network16SmallSort
end

component sk_network16
  type     Network16SmallSort
  label    network: 16
  provides SmallSortKind
  category SmallSort
  max_input 16
  uses     small_sort_network_16::Network16SmallSort
end

