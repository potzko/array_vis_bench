# rotations

One crate per rotation. Each implements `Rotation`, declares any scratch it
needs through `scratch_size`, and provides the `Rotation` role, which is both
the slot of a rotation merge and the first class rotation entry.

- `rotation_auxiliary`: copy the smaller side to a buffer.
- `rotation_bridge`: buffer sized to the difference of the sides.
- `rotation_contrev`: conjoined triple reversal.
- `rotation_drill`: grail, piston and helix loops combined.
- `rotation_grail`: Gries Mills with a small stack buffer.
- `rotation_gries_mills`: repeated block swaps.
- `rotation_helix`: grail derived with alternating loops.
- `rotation_juggling`: gcd cycles.
- `rotation_piston`: successive block swaps.
- `rotation_reversal`: triple reversal.
- `rotation_trinity`: contrev plus bridge with an eight element scratch.
