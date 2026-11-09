# `rotation_piston_opt`

Optimized piston rotation: piston with the middle of the range run as a ring,
after Stew Forster's triple shift rotate (2025), reimplemented from a
description of it.

Piston swaps the smaller side with the window of its length at the far end of
the larger side, three moves a column, and only that window lands in place.
Here each column also passes through the cells between the two windows, used
as a ring that delays every element by the distance it sits from its place, so
the near window lands in place as well: four moves a column put two elements
where piston's three put one, and the range closes in from both ends onto the
ring. A range whose smaller side, or whose difference of sides, fits a 256
cell buffer is finished through it.

`OptimizedPiston<WINDOWS, BUFFER>` takes the windows a pass cycles and the
buffer. Its catalog components, declared in `rotation_piston_opt.spec`:

- `optimized piston`: two windows and 256 cells, `OptimizedPistonRotation`.
- `optimized piston<in place>`: two windows and no buffer. The buffered one
  runs no pass below 771 elements, so this is the trace that shows the ring.
- `optimized piston<3 windows>`: three windows and 256 cells.

## Two axes

A window is a run as long as the smaller side. A pass cycles the smaller
side's window, whole windows of the larger side next to it and the window at
the far end, through one temporary a column, so `k` windows cost `k + 1`
moves a column and land `k - 1` of them in place. The rest of the larger side,
between those windows, either waits for the next pass or runs as a ring, one
more stream in the cycle. The two choices are independent:

| windows | ring | moves, n = 1000 | n = 4096 | |
|---|---|---|---|---|
| 2 | no | 2.98n | 3.00n | piston |
| 3 | no | 2.29n | 2.31n | |
| all | no | 1.83n | 1.84n | block cycle's count |
| 2 | yes | 1.99n | 2.00n | `optimized piston` |
| 3 | yes | 1.78n | 1.79n | `optimized piston<3 windows>` |
| 4 | yes | 1.69n | 1.70n | |
| all | yes | 1.55n | 1.55n | a ring block cycle |

Each count is the average over every split, in place, and agrees with the
description's own simulation. With the ring no split needs more than 2n.

Piston is the point with two windows and no ring, but it keeps its own crate.
The driver here, run at that point, logged piston's exact trace and took 5 to
10 percent longer on splits made of many short passes: a thousand elements
split at 499 took 0.560 ns per element against piston's 0.505.

Four windows and more need more streams a column than the three and four this
crate moves column by column, and batching them through the buffer, as block
cycle does, cost more than the moves saved: four windows took 0.100 ns per
element over all splits of a hundred thousand `u32`, three took 0.070. The
ring block cycle, batched the same way, was the fastest of all for a million
`u64`, 0.240 against 0.263 for three windows, but took 0.094 for a hundred
thousand `u32`.

- `src`: the crate's source.
