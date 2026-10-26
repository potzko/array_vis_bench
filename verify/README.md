# `verify`

The baselines and scripts behind `../verify.sh`, the one command that proves
a change kept the catalog, the generated code, the correctness batteries and
the performance where they were.

- `catalog.types`: every emitted entry's display name and concrete Rust type,
  the companion of `spec_catalog/catalog.golden`, which pins category, name
  and complexity but cannot see a swapped type.
- `machine`: the rustc version, target cpu and cpu model the assembly and
  timing baselines were taken on. The two comparisons below are skipped with
  a failure when it differs, because both are machine specific.
- `asm/`: the normalised disassembly of the six `compare_sorts` gate symbols,
  three hand rolled sorts and the three generic ones they are measured
  against, as `asm_text.sh` prints them.
- `timing.tsv`: the median run time of every entry at size 1000 over
  fifteen interleaved rounds, as `array_vis_bench --timing 1000 15 --out`
  writes it. Interleaving is what makes the medians stable: a burst of
  machine load lands on different entries in different rounds.
- `fingerprint.py`: produces `catalog.types` from the generated
  `catalog_entries.rs`.
- `compare_timing.py`: compares a timing archive with the baseline. One entry
  regresses when it is slower by more than thirty percent and by more than
  fifty microseconds; the whole catalog regresses when the sum of the shared
  medians grows by more than ten percent. Measured run to run noise with
  interleaved rounds is about four percent at the 99th percentile, and the
  machine itself drifts by about five percent between days, which is what
  the total gate has to leave room for. The drift free measurement is the
  `compare_sorts` gate, which times a generic sort against a hand written one
  in the same run.
- `test_compare_timing.py`: the comparison's unit tests, which the gate runs.

When a change is meant to move a baseline, review the diff the gate printed,
then run `../verify.sh --bless` and commit the new baseline with the change.
Timing noise on a busy machine can fail the timing step once; rerun before
treating it as a regression.
