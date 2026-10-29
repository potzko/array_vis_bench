# Implementation plan: instruction counts for the code generation gate

Status: NOT STARTED. Written 2026-10-29.

## Problem

The zero cost claim is that a composition the spec compiler emits compiles to
the code a person would write by hand. Today it is checked two ways: the
assembly listings of three hand written sorts and their generic twins are
diffed against baselines, and `compare_sorts` times the pairs and allows
1.05x. The listings catch a changed instruction stream but say nothing about
how often each instruction runs, and the timing allows five percent because
timing is noisy, not because five percent is the claim.

An instruction count is the measurement in between: deterministic to the
instruction on a fixed input, so it can be pinned exactly, and sensitive to
what the code does rather than how it is laid out. It also reaches the whole
framework rather than three pairs: the adapter each kind emits, the no op
logger path, the dispatch through `run_with_input`, all of which must add
nothing per call.

## Design

`iai-callgrind` (MIT or Apache 2) runs a function under valgrind's callgrind
and reports its instruction count, `Ir`, with the cache simulation off. It
drives valgrind as a separate process; valgrind is GPL 2 and is a tool on the
machine, not a dependency of the code, so the licence check stops at the crate.
The library and its runner binary must be the same version.

One bench target, `benches/instructions.rs`, `harness = false`, with:

- the six `compare_sorts` symbols on the generator and size that binary uses,
  each a `#[library_benchmark]` taking its input from a setup function so the
  input's cost is outside the count;
- one entry per kind of the catalog at size 1000 through its
  `run_with_input` under the no op logger, so every emitted adapter shape is
  counted.

A gate step `instructions` in `verify.sh`, after `asm`:

- runs the bench with JSON output into `target/verify/instructions.json`;
- compares every count to `verify/instructions.json` exactly, since a count
  on a fixed input is deterministic; `--bless` rewrites the baseline after a
  change meant to move it;
- for each of the three pairs fails when the generic count exceeds the hand
  count by more than two percent, the margin the listings already show, one
  saved register and one bounds check;
- is skipped with a FAIL line when valgrind or the runner is missing, the
  way `asm` is skipped when the machine stamp differs, so a machine without
  valgrind sees the gap and the rest of the gate still runs.

The counts depend on the rustc version and flags, which the machine stamp
already records, and not on the cpu, so the step may compare across
machines; the stamp check stays as it is.

## Risks

- `target-cpu=native` may emit instructions valgrind does not implement. This
  cpu has no AVX-512, and valgrind covers AVX2, so the sorts should run; if a
  symbol trips valgrind, build the bench target with a baseline target cpu and
  say so in the baseline file's header.
- A callgrind run is about fifty times slower than native, so the inputs
  stay at the sizes above and the step takes under a minute.
- The runner version must match the crate version; the gate prints both when
  they differ.

## Steps

1. Install valgrind (`apt install valgrind`) and the runner
   (`cargo install iai-callgrind-runner --version <the crate's version>`);
   record both in `verify/README.md` beside the machine stamp.
2. The bench target and a first run by hand; read the six pair counts and
   confirm they match what the listings suggest.
3. The gate step, test first: the comparison is a small Python script beside
   `compare_timing.py` with its own unit tests, reading two JSON files and
   printing one line per benchmark.
4. Bless the baseline, run the full gate, then the docs: the command table in
   the root readme, `verify/README.md`, item 10 of `../todo.md`.

## Pointers

- The pairs: `src/bin/compare_sorts.rs`, the symbols `asm_text.sh` lists.
- The kinds: the `KINDS` table in `spec_emit`, `primary_input` and
  `run_with_input` in `array_vis_bench_core::bench_registry`.
- The gate: `verify.sh`, `verify/README.md`, `verify/compare_timing.py` as
  the shape of the comparison script.
