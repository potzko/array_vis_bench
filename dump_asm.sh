#!/usr/bin/env bash
# Dump release ASM for one or more symbols into target/asm/<symbol>.s.
#
# Builds compare_sorts in release first, then resolves each symbol through
# `cargo-show-asm`. Pair with the `hand_*` / `sys_*_u64` wrappers in
# src/bin/compare_sorts.rs and any `asm_*` variants opted in via a family's
# `asm = true` Cargo.toml knob.
#
# Symbols that share a name with multiple monomorphisations get
# disambiguated through the `compare_sorts` binary's own export table: the
# `sys_*_u64` wrappers do a `jmp` to the exact mono the bench measures, so
# objdump that wrapper to find the mangled callee, then pass it here.
#
# Usage:
#   ./dump_asm.sh hand_quicksort sys_quicksort_u64
#   ./dump_asm.sh hand_quicksort | less
#
# After dumping, diff with:
#   diff -u target/asm/hand_quicksort.s target/asm/sys_quicksort_u64.s

set -euo pipefail

if [ $# -eq 0 ]; then
  echo "usage: $0 <symbol> [<symbol> ...]" >&2
  exit 1
fi

mkdir -p target/asm
cargo build --release --bin compare_sorts >/dev/null

for sym in "$@"; do
  out="target/asm/${sym}.s"
  if cargo asm -p array_vis_bench --bin compare_sorts --release --simplify "$sym" >"$out" 2>/dev/null; then
    if [ -s "$out" ]; then
      echo "wrote $out ($(wc -l <"$out") lines)"
    else
      echo "no asm produced for $sym — try the mangled name from objdump" >&2
      rm -f "$out"
    fi
  else
    echo "cargo asm failed for $sym" >&2
    rm -f "$out"
  fi
done
