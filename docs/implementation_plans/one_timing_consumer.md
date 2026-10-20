# Implementation plan: one timing consumer

Status: DONE (2026-10-20). Designed from the concept review recorded in
`../todo.md`. Feeds todo item 5, the persisted timing archive. Built as
designed, with three choices the design left open. The report's `ran` counts
the entries that ran and leaves out the skipped ones, whose count is printed
in the benchmark's header. The tables come out in category name order from
the entries' own categories, so there is no category list to extend and the
kind recipe in `docs/registration.md` lost its last step.
`spec_catalog::run_timing` takes `Option` overrides and runs the whole
catalog. At the default `n` of 1024 the benchmark skips 187 of the 1580
entries, 117 of them sorts capped at 1000. The catalog golden did not move
and the emitted entries are identical.

## Problem

Four surfaces timed or exercised entries.

- `--timing N K` in `src/main.rs` called `spec_catalog::run_timing`, which
  ran the `Timing` main over the Sort category and printed the ten fastest
  and ten slowest.
- A binary under `src/bin` had its own loop: every category, median of the
  runs, a full fastest first table per category. It was not a `Main`, so
  `main_by_name` could not reach it.
- `Benchmark`, the `Main` named `benchmark`, ran each entry once under
  `NoOpLogger` inside `catch_unwind` and reported panics. It measured nothing.
  `--consumers` and `consumers.avbs` called it benchmark.
- `compare_sorts` is the regression gate against hand written baselines. It
  is a different thing and stays.

`Timing` was not reachable by name either: `main_by_name` knew visualiser,
correctness and benchmark.

## Design

One timing consumer, named `benchmark`: the body of today's `Timing` with the
reporting of the speed test binary. It groups by category, prints the full
table of medians over `k` runs, skips and counts entries whose cap is below
`n`, and has `n` and `k` fields with defaults of 1024 and 3. It is registered
in `main_by_name`, so `consumers.avbs` reaches it.

Today's `Benchmark` goes. The correctness consumer runs the same code under
the same logger over a wider set of inputs, so the smoke run adds nothing.

`--timing [N] [K]` stays as the command line door and overrides the defaults.
The speed test binary is deleted.

`MainReport` gains `timings: Vec<(&'static str, Duration)>`. That is the seam
the persisted archive needs: a later step writes it as JSON and
`plot_bench.py` gets its producer back.

## Steps

1. Tests first. `shells_tick_through_the_consumers` and
   `consumers_binding_drives_the_mains_end_to_end` in `spec_catalog` already
   expect three reports including `benchmark`; add the assertion that the
   benchmark report carries one timing per entry that ran. In `mains`, a test
   that `main_by_name("benchmark")` has the defaults.
2. Move the per category table from the speed test binary into `Timing`,
   rename it `Benchmark`, delete the old one, register it by name, add
   `timings` to the report.
3. Point `--timing` at it through `run_timing`, delete the speed test binary,
   update `src/bin/README.md`, the command table in the root README and item
   7 of the kind list in `docs/registration.md`.

## Checks

- `cargo run --release -- --timing 10000 5` prints one table per category.
- `src/bin` holds `compare_sorts` and `measure_random_shell` only.
- `cargo run --release -- --consumers` still runs correctness and benchmark
  over the full catalog and exits non zero on a failure.
