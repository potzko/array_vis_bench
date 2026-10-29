#!/usr/bin/env bash
# The branch policy behind the hooks, `promote.sh` and `verify.sh`: the level
# of gate each shared branch needs, and the stamps that say a tree passed one.
# A stamp is a file named after a tree sha under the common git dir, so every
# worktree of a clone sees it, and holds the level and the `rustc -V` of the
# run. Keyed by tree, it survives a rebase that leaves the content unchanged.
#
#   gate.sh clean-tree            HEAD's tree sha, nothing when the checkout is dirty
#   gate.sh stamp LEVEL TREE      record a pass, TREE being clean-tree's answer at the start
#   gate.sh check REV LEVEL       whether REV's tree carries a stamp of LEVEL or above
#   gate.sh level BRANCH          the level BRANCH needs, nothing when it is unguarded
set -uo pipefail

STAMPS="$(git rev-parse --path-format=absolute --git-common-dir)/verify-stamps"

usage() {
  sed -n '/^#   gate.sh/s/^#   /usage: /p' "$0" >&2
  exit 2
}

level() {  # <branch>
  case "$1" in
    dev) echo portable ;;
    main) echo full ;;
  esac
}

command_for() {  # <level>
  if [[ $1 == portable ]]; then echo "./verify.sh --portable"; else echo "./verify.sh"; fi
}

clean_tree() {
  [[ -z $(git status --porcelain --untracked-files=normal) ]] && git rev-parse 'HEAD^{tree}'
}

stamp() {  # <level> <tree at the start of the run>
  local file="$STAMPS/$2"
  if [[ -z $2 ]]; then echo "stamp: none, the tree was not clean when the run started"; return; fi
  if [[ $(clean_tree) != "$2" ]]; then echo "stamp: none, the tree changed during the run"; return; fi
  if [[ $1 == portable && $(head -1 "$file" 2>/dev/null) == full ]]; then echo "stamp: tree ${2:0:7} keeps its full stamp"; return; fi
  mkdir -p "$STAMPS" && printf '%s\n%s\n' "$1" "$(rustc -V)" >"$file" && echo "stamp: $1 on tree ${2:0:7}"
}

check() {  # <rev> <level>
  local tree have toolchain name
  tree=$(git rev-parse --verify -q "$1^{tree}") || { echo "$1 is not a commit" >&2; return 1; }
  name="$(git rev-parse --short "$1") (tree ${tree:0:7})"
  if [[ ! -f $STAMPS/$tree ]]; then
    echo "no stamp on $name: run $(command_for "$2") in a clean checkout of it" >&2
    return 1
  fi
  { read -r have; read -r toolchain; } <"$STAMPS/$tree"
  if [[ $2 == full && $have != full ]]; then
    echo "$name has a $have stamp and needs a full one: run ./verify.sh in a clean checkout of it" >&2
    return 1
  fi
  if [[ $2 == full && $toolchain != "$(rustc -V)" ]]; then
    echo "$name passed under $toolchain, the toolchain is now $(rustc -V): run ./verify.sh again" >&2
    return 1
  fi
}

is_level() { [[ $1 == portable || $1 == full ]]; }

case "${1:-}" in
  clean-tree) [[ $# -eq 1 ]] || usage; clean_tree ;;
  stamp) [[ $# -eq 3 ]] && is_level "$2" || usage; stamp "$2" "$3" ;;
  check) [[ $# -eq 3 ]] && is_level "$3" || usage; check "$2" "$3" ;;
  level) [[ $# -eq 2 ]] || usage; level "$2" ;;
  *) usage ;;
esac
