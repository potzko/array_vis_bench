#!/usr/bin/env bash
# Side-by-side disassembly of two symbols in the compare_sorts binary.
#
#   ./asmdiff.sh quick            # hand_quicksort  vs  sys_quicksort_u64
#   ./asmdiff.sh merge            # hand_mergesort  vs  sys_mergesort_u64
#   ./asmdiff.sh heap
#   ./asmdiff.sh <sym_a> <sym_b>  # any two #[no_mangle] symbols
#
# The listings come from `asm_text.sh`, which follows a `sys_*` wrapper's
# single jmp into the monomorphised generic and strips every unstable byte.
set -euo pipefail

BIN=${BIN:-target/release/compare_sorts}

# Shorthand: one arg naming a sort pair -> hand_<x>sort vs sys_<x>sort_u64.
if [[ $# -eq 1 ]]; then
  A="hand_${1}sort"; B="sys_${1}sort_u64"
elif [[ $# -eq 2 ]]; then
  A="$1"; B="$2"
else
  echo "usage: $0 <quick|merge|heap> | <symA> <symB>" >&2; exit 2
fi

cargo build --release --bin compare_sorts -q

dump() { BIN="$BIN" ./asm_text.sh "$1"; }

OUT=${OUT:-asmdiff.txt}
{
  echo "===== HAND-ROLLED: $A ====="
  dump "$A"
  echo
  echo "===== SYSTEM: $B ====="
  dump "$B"
} > "$OUT"
echo "wrote $OUT"
