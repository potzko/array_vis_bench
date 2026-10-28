#!/usr/bin/env bash
# Print the normalised disassembly of one symbol in the compare_sorts binary.
# `asmdiff.sh` pairs two of these side by side and `verify.sh` diffs one
# against its committed baseline, so the text here must be stable across
# builds that did not change the code: addresses, raw bytes, assembler
# comments, symbol offsets, call targets' addresses, rip relative offsets,
# the alignment padding and, in mangled names, the hash suffix and the crate
# id are stripped. The crate id of a binary changes whenever its dependency
# set does, so a crate added anywhere in the workspace would otherwise show
# as a change here; the padding is the `nop` run the alignment flags in
# `.cargo/config.toml` put before every loop header.
#
#   ./asm_text.sh hand_quicksort
#   BIN=target/release/compare_sorts ./asm_text.sh sys_quicksort_u64
#
# A `sys_*` wrapper is a lone `jmp` into the monomorphised generic; that
# jump is followed so the listing is the real function.
set -euo pipefail

BIN=${BIN:-target/release/compare_sorts}
[[ $# -eq 1 ]] || { echo "usage: $0 <symbol>" >&2; exit 2; }
sym=$1

listing=$(objdump -d -M intel --no-show-raw-insn "$BIN" --disassemble="$sym")
target=$(grep -oP '^\s+[0-9a-f]+:\s+jmp\s+[0-9a-f]+ <\K[^>]+' <<<"$listing" | head -1 || true)
if [[ -n "$target" && $(grep -cE '^\s+[0-9a-f]+:' <<<"$listing") -eq 1 ]]; then
  listing=$(objdump -d -M intel --no-show-raw-insn "$BIN" --disassemble="$target")
fi
sed -E 's/^\s+[0-9a-f]+:\s+//; /^[0-9a-f]+ </d; s/\s*#.*//;
        s/[0-9a-f]+ <([^>]*)\+/<\1+/g; s/<[^>]*\+/<+/g; s/17h[0-9a-f]{16}E/E/g;
        s/ [0-9a-f]+ </ </g; s/\[rip\+0x[0-9a-f]+\]/[rip+?]/g;
        s/Cs[0-9A-Za-z]+_([0-9]+)/Cs_\1/g;
        /^(data16 |cs )*nop/d;
        /^$/d; /Disassembly of/d; /file format/d' <<<"$listing"
