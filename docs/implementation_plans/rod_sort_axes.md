# Implementation plan: rod sort axes

Status: done 2026-10-29, kept as the record; the tuned strategy was dropped,
see its section. Written after the rod sort review that dropped its stability
claim, gave its time bounds to the strategy and merge together and fixed the
chain length the strategy is told.

## What rod sort is

Rod sort recurses into the `branch` chains of a slice at stride `jump *
branch`, then merges the sorted chains back at stride `jump`. The merge decides
which sort it is.

- With `InsertionMerge` it is a Shell sort. A level with jump `j` is a Shell
  pass with gap `j`, the `inter` pass one with gap `j * inter`, and the depth
  first order only reorders chains that never meet. Each jump divides the next,
  the classic weakness of a gap sequence, and `inter` is the only gap off that
  chain.
- With `AuxMerge` it is a merge sort over a fixed permutation of the input.
  Each subtree holds a residue class instead of a block, and the comparisons
  are those of a merge sort run on the digit reversed input.

So the ceilings are a good Shell sort and a merge sort. From a probe on 2^20
random `u64`, one P core, nanoseconds per element:

| variant | ns |
|---|---|
| `classic` with `AuxMerge` | 191 |
| `optimised` with `AuxMerge` | 322 |
| `root-parity` with `AuxMerge` | 3190 |
| `optimised` with `InsertionMerge` | 247 |
| Shell sort, Ciura gaps | 94 |
| top down merge sort, ping pong, insertion leaves of 16 | 58 |

## Not an axis: a transposed layout

Gathering each leaf's chain into a contiguous run of the aux buffer once, then
merging contiguous runs in ping pong, ran at 80 ns. With a cut at 16 it matched
the top down merge sort to 2^17 and trailed it by 10 to 17 percent from 2^20 to
2^22. It is merge sort after a permutation: the gather is the permutation and
the rest is a top down merge sort, k way for a wider strategy. The in place
form, unshuffling each node by rotations and merging by a `RotationMerge`,
reduces the same way. Every unshuffle in a subtree comes before every merge in
it and touches only its own segment, so all of them hoist to the front: a
permutation, then a rotation merge sort.

Against the merge sort family such a sort keeps three things.

- The pairing of elements in each merge. On random input this changes nothing,
  since a fixed permutation of a random input is random. On structured input it
  costs the adaptivity: on sorted input at 2^16 it compared 0.86 n log2 n
  times, the top down merge sort 0.43 and the early exit one 0.06.
- The permutation pass, the 10 to 17 percent above.
- Trees wider than two, with a fan in that may grow with the length. That idea
  is real, a multiway merge sort over a tournament merge, and belongs to the
  merge sort family as its own axis, outside this plan.

What makes rod sort itself is the strided layout, in memory and on screen.
Everything below keeps it.

## Fixes that come first

- `insertion_sort_jump` holds the element and shifts its chain predecessors up
  instead of swapping at every step. The comparisons stay the same, writes go
  from 7.5 to 4.0 per n log2 n, and `optimised` with `InsertionMerge` from 247
  to 158 ns. The visual shows writes where it showed swaps.
- `AuxMerge` stops allocating its heads on every call. `RodMerge` declares
  `aux_size(n)` and `index_size(branch)` in place of a flag, and the driver
  creates each buffer once through the logger, the way the rotations' scratch
  works, so the heads are an index array the visualiser shows moving. The
  index is sized by the top level's branch, the widest while a strategy's
  branch grows with the length, and a merge handed less creates its own for
  that call. `optimised` went from 322 to 191 ns, `root-parity` from 3190 to
  892.

## The merge axis: a tournament merge

`TournamentMerge` is a third `RodMerge` filler. It keeps the chain heads in a
loser tree, reads them at stride `jump` into the shared aux buffer, and copies
back as `AuxMerge` does. Each output costs ceil(log2 k) comparisons for k
chains. Over a root to leaf path the logs of the branches sum to log2 N, so the
sort is N log N under every strategy and its bound needs no strategy constant.
Two chains take a plain two pointer loop with the heads in locals, mirrored
into the index for the visualiser: `classic` took 227 ns through the tree,
185 reading the heads back from the index, 139 like this.

The prototype compared 0.92 to 1.0 n log2 n times at 2^16 under all six
strategies, where the scan compares 0.92 to 17. Built, it ran the wide
strategies in 121 to 135 ns at 2^20 against 141 to 892 for the scan.

Its test runs one merge over k sorted chains of n elements in total under a
counting logger and holds it to n ceil(log2 k) plus k comparisons, for k from
2 to 40. The catalog battery covers sortedness. The golden gains six
`rod sort<strategy: *, merge: tournament>` entries, N log N throughout, space N.

## The order axis: Shell sort over rod gaps

The depth first recursion of the insertion path works through one chain at a
time, and chains that share a cache line are visited far apart. Running each
gap as one Shell pass over every chain makes the same comparisons in another
order: 132 against 158 ns with the shift kernel, 217 against 249 with swaps.

This needs no new driver. `RodGaps<S>` implements
`shell_sort_lib::GapSequence` from a strategy's gap chain, read off the
leftmost path: each level's jump, then that jump times `inter` where the length
gate holds, down to the cut. It lives in `rod_sort_lib`, which takes a
dependency on `shell_sort_lib` for the trait, and `rod.spec` declares it as a
`GapSequence` filler with a `strategy` slot. The shell queries' `_` reaches it
unchanged, so both shell drivers gain one entry per strategy. Its bounds are the
insertion path's: N^2 worst and average, N times the strategy's depth best.

Where the tree is uniform, every node at a depth holding the same length, the
passes are exactly those of `RodSort<S, InsertionMerge>`. Elsewhere a sibling
that cuts a level earlier is the difference. The test runs both on the same
input at powers of the branch for `classic`, `3-parity` and `optimised`, and
asserts equal comparison counts.

Built, these entries run on `ShellSort`'s kernel, which swaps at every step,
while rod sort's own insertion now shifts: `optimised` at 2^20 took 322 ns
through the shell driver against 231 for the recursion. On one kernel the pass
order is the faster by about a tenth, 215 against 247 shifting and 327
against 362 swapping, so the hold and shift kernel in the shell family is what
would make this axis pay.

## Dropped: a strategy for the insertion path

The search at 2^16 put branch 10 with `inter` 9 first. Rerun at 2^20 the best
constant setting moved to branch 17 with `inter` 16, 5.51 n log2 n, and
`log-parity`, whose branch grows with the length and whose `inter` is its
branch less one, beat every constant setting at both sizes: 2.86 against 2.97
at 2^16, 5.26 against 5.51 at 2^20. A constant strategy would join already
dominated. Ciura read 1.53 and 1.60, and the insertion path's comparisons per
n log2 n roughly double from 2^16 to 2^20, the divisor chain showing.

## Left out

- A `SmallSort` leaf. In the strided layout each leaf is gathered into a
  buffer, sorted and scattered back, and the cut moves from the strategy to the
  leaf's threshold, which changes what every existing strategy means.
- A list of `inter` passes per level, the Ciura gaps below the branch. It cut
  comparisons at 2^16, to 2.0 n log2 n for branch 64, but at 2^20 it read 3.3
  against Ciura's 1.6. The divisor chain stays, and the gain does not pay for
  the change to `BranchingStrategy`.

## Catalog and docs

Rod sorts went from 12 entries to 18, six strategies by three merges. Shell
sorts went from 18 to 30, nine sequences and six rod gap chains by two
drivers. The catalog went from 1868 to 1886. The full gate's timing baseline
gains rows for the new entries by hand, never by `--bless`.

## Follow up

The hold and shift kernel for `ShellSort` and `ShellSortOrdered`: the same
comparisons, fewer writes, writes instead of swaps on screen, and the rod gap
entries ahead of the recursion they mirror.
