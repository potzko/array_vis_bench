#!/usr/bin/env bash
# The verification gate: one command that proves a change kept the catalog,
# the generated code, the correctness batteries and the performance where
# they were. Each step prints one PASS or FAIL line and the run exits non
# zero if any step failed. Baselines live under `verify/`; `--bless` rewrites
# them after a change that was meant to move them.
#
#   ./verify.sh            every step, several minutes
#   ./verify.sh --quick    skip the workspace tests and the timing step
#   ./verify.sh --bless    run every step and rewrite the baselines it owns
#
# Steps, in order:
#   build     release build of the workspace, no warnings allowed
#   catalog   the spec compiler's own tests and golden, then the emitted
#             catalog's names and types against `verify/catalog.types`
#   scripts   the unit tests of the comparison script
#   machine   the rustc version, target cpu and cpu model this machine's
#             assembly and timing baselines were taken on
#   asm       the six gate symbols of `compare_sorts` against `verify/asm/`
#   perf      the `compare_sorts` gate, generic sorts within 1.05x of hand
#             rolled ones
#   tests     `cargo test --release --workspace`
#   timing    every entry timed and compared against `verify/timing.tsv`
set -uo pipefail
cd "$(dirname "$0")"

QUICK=0
BLESS=0
for arg in "$@"; do
  case "$arg" in
    --quick) QUICK=1 ;;
    --bless) BLESS=1 ;;
    *) echo "usage: $0 [--quick] [--bless]" >&2; exit 2 ;;
  esac
done

BASE=verify
OUT=target/verify
mkdir -p "$OUT/asm" "$BASE/asm"
FAILED=0
pass() { echo "PASS: $1"; }
fail() { echo "FAIL: $1"; FAILED=1; }
bless_or_diff() {  # <label> <baseline> <current>
  if [[ $BLESS -eq 1 ]]; then
    cp "$3" "$2" && pass "$1 (baseline written)"
  elif [[ ! -f "$2" ]]; then
    fail "$1: no baseline at $2, run with --bless"
  elif diff -q "$2" "$3" >/dev/null; then
    pass "$1"
  else
    fail "$1: $(diff "$2" "$3" | grep -c '^[<>]') lines differ, see diff $2 $3"
  fi
}

echo "== build"
if cargo build --release >"$OUT/build.log" 2>&1 && ! grep -q '^warning' "$OUT/build.log"; then
  pass "release build, no warnings"
else
  fail "release build ($(grep -c '^warning\|^error' "$OUT/build.log") warnings or errors, see $OUT/build.log)"
fi

echo "== catalog"
if cargo test --release -p spec_catalog >"$OUT/catalog_tests.log" 2>&1; then
  pass "spec_catalog tests and golden ($(grep -oE '^test result: ok\. [0-9]+' "$OUT/catalog_tests.log" | awk '{s+=$4} END{print s}') tests)"
else
  fail "spec_catalog tests (see $OUT/catalog_tests.log)"
fi
ENTRIES=$(ls -t target/release/build/spec_catalog-*/out/catalog_entries.rs | head -1)
python3 -I "$BASE/fingerprint.py" "$ENTRIES" >"$OUT/catalog.types"
bless_or_diff "emitted catalog names and types ($(wc -l <"$OUT/catalog.types") entries)" "$BASE/catalog.types" "$OUT/catalog.types"

echo "== scripts"
if (cd "$BASE" && python3 -m unittest -q test_compare_timing >"../$OUT/scripts.log" 2>&1); then
  pass "verify script tests"
else
  fail "verify script tests (see $OUT/scripts.log)"
fi

echo "== machine"
{
  rustc -V
  grep -o 'target-cpu=[a-z0-9-]*' .cargo/config.toml
  lscpu | grep 'Model name' | sed 's/.*: *//'
} >"$OUT/machine"
bless_or_diff "toolchain and cpu stamp" "$BASE/machine" "$OUT/machine"
SAME_MACHINE=$([[ -f "$BASE/machine" ]] && diff -q "$BASE/machine" "$OUT/machine" >/dev/null && echo 1 || echo 0)

echo "== asm"
if [[ $SAME_MACHINE -eq 1 ]]; then
  for sym in hand_quicksort sys_quicksort_u64 hand_mergesort sys_mergesort_u64 hand_heapsort sys_heapsort_u64; do
    ./asm_text.sh "$sym" >"$OUT/asm/$sym.s"
    bless_or_diff "assembly of $sym ($(wc -l <"$OUT/asm/$sym.s") lines)" "$BASE/asm/$sym.s" "$OUT/asm/$sym.s"
  done
else
  fail "assembly not compared: the machine stamp differs, review and --bless"
fi

echo "== perf"
if ./target/release/compare_sorts >"$OUT/perf.log" 2>&1; then
  pass "compare_sorts gate, quick merge heap medians: $(grep -oE 'median: [0-9.]+' "$OUT/perf.log" | sed 's/median: //' | tr '\n' ' ')"
else
  fail "compare_sorts gate (see $OUT/perf.log)"
fi

if [[ $QUICK -eq 1 ]]; then
  echo "== tests and timing skipped (--quick)"
else
  echo "== tests"
  if cargo test --release --workspace >"$OUT/tests.log" 2>&1; then
    pass "workspace tests ($(grep -oE '^test result: ok\. [0-9]+' "$OUT/tests.log" | awk '{s+=$4} END{print s}') tests)"
  else
    fail "workspace tests ($(grep -cE '^test .* FAILED|panicked' "$OUT/tests.log") failures, see $OUT/tests.log)"
  fi

  echo "== timing"
  ./target/release/array_vis_bench --timing 1000 3 --out "$OUT/timing.tsv" >"$OUT/timing.log" 2>&1
  if [[ $BLESS -eq 1 ]]; then
    cp "$OUT/timing.tsv" "$BASE/timing.tsv" && pass "timing baseline written ($(wc -l <"$BASE/timing.tsv") entries)"
  elif [[ $SAME_MACHINE -ne 1 ]]; then
    fail "timing not compared: the machine stamp differs, review and --bless"
  elif python3 -I "$BASE/compare_timing.py" "$BASE/timing.tsv" "$OUT/timing.tsv" >"$OUT/timing_compare.log" 2>&1; then
    pass "timing: $(head -1 "$OUT/timing_compare.log")"
  else
    fail "timing regressed (see $OUT/timing_compare.log)"
    sed -n '2,8p' "$OUT/timing_compare.log"
  fi
fi

echo
if [[ $FAILED -eq 0 ]]; then echo "VERIFY: all steps passed"; else echo "VERIFY: a step failed"; fi
exit $FAILED
