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

Its catalog components are declared in `rotation_piston_opt.spec`.

- `src`: the crate's source.
