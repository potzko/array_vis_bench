# cycle_sorts

Cycle sort — the theoretically optimal sort for minimising the number of writes to the array.

## How cycle sort works

For each element, cycle sort counts how many elements are smaller — that count is the element's correct final position. It then places the element there, displacing whatever was in that position, and repeats until it returns to the starting position (completing one "cycle"). Each element is written to its final position exactly once.

This makes cycle sort optimal for writes (exactly N writes for N elements not already in place) but O(N^2) for comparisons, since finding each element's position requires scanning the array.

## When cycle sort matters

Write-minimality is important when writes are expensive (e.g. flash memory, EEPROM) or when you want to visualise the minimum-write sorting pattern.

## Files

- `cycle_sort.rs` — standard cycle sort implementation.

## Registration

Single-leaf `combo_codegen::family!` invocation in `cycle_sort.rs`.
