#!/usr/bin/env bash
# Side-by-side disassembly of two symbols in the compare_sorts binary.
#
#   ./asmdiff.sh quick            # hand_quicksort  vs  sys_quicksort_u64
#   ./asmdiff.sh merge            # hand_mergesort  vs  sys_mergesort_u64
#   ./asmdiff.sh heap
#   ./asmdiff.sh <sym_a> <sym_b>  # any two #[no_mangle] symbols
#
# sys_* wrappers are 5-byte tail-jumps into the monomorphised generic; this
# follows that single jmp automatically so you see the real function.
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

# Disassemble one symbol; if its body is a lone `jmp <target>`, follow it.
# Strip addresses / raw bytes / hash-comments and blank the differing symbol
# name inside jump targets so the two columns line up under diff.
dump() {
  local sym=$1
  local out; out=$(objdump -d -M intel --no-show-raw-insn "$BIN" --disassemble="$sym")
  local target; target=$(grep -oP '^\s+[0-9a-f]+:\s+jmp\s+[0-9a-f]+ <\K[^>]+' <<<"$out" | head -1)
  if [[ -n "$target" && $(grep -cE '^\s+[0-9a-f]+:' <<<"$out") -eq 1 ]]; then
    out=$(objdump -d -M intel --no-show-raw-insn "$BIN" --disassemble="$target")
  fi
  sed -E 's/^\s+[0-9a-f]+:\s+//; /^[0-9a-f]+ </d; s/\s*#.*//;
          s/[0-9a-f]+ <([^>]*)\+/<\1+/g; s/<[^>]*\+/<+/g;
          /^$/d; /Disassembly of/d; /file format/d' <<<"$out"
}

OUT=${OUT:-asmdiff.txt}
{
  echo "===== HAND-ROLLED: $A ====="
  dump "$A"
  echo
  echo "===== SYSTEM: $B ====="
  dump "$B"
} > "$OUT"
echo "wrote $OUT"
