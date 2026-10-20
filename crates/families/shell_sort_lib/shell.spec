# Shell-sort family catalog fragment — OWNED by this crate.
#
# The spec compiler (`spec_catalog`) gathers this fragment via
# `[package.metadata.array_vis_bench] spec = "shell.spec"` and merges it into the
# unified registry. It declares the {ShellSort, ShellSortOrdered} drivers and the
# 9 gap sequences (all defined in this crate). `uses` paths resolve in the
# consumer (`spec_catalog`), which links this crate for its types.

component shell_sort
  type     ShellSort<{seq}>
  label    shell sort<sequence: {seq}>
  provides Sort
  menu     shell sorts / shell sort
  slot     seq GapSequence
end

component shell_sort_ordered
  type     ShellSortOrdered<{seq}>
  label    shell sort ordered<sequence: {seq}>
  provides Sort
  menu     shell sorts / shell sort ordered
  slot     seq GapSequence
end

# Gap sequences (9) — labels match `GapSequence::NAME`.
component classic
  type     ClassicGaps
  label    classic
  provides GapSequence
end

component knuth
  type     Knuth
  label    knuth
  provides GapSequence
end

component hibbard
  type     Hibbard
  label    hibbard
  provides GapSequence
end

component sedgewick
  type     Sedgewick
  label    sedgewick
  provides GapSequence
end

component sedgewick_branching
  type     SedgewickBranching
  label    sedgewick-branching
  provides GapSequence
end

component ciura
  type     Ciura
  label    ciura
  provides GapSequence
end

component tokuda
  type     Tokuda
  label    tokuda
  provides GapSequence
end

component pratt
  type     Pratt
  label    pratt
  provides GapSequence
end

component optimized256
  type     Optimized256
  label    optimized-256
  provides GapSequence
end
