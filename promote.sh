#!/usr/bin/env bash
# Promotion onto the shared branches. `./promote.sh BRANCH` fast forwards
# `dev` to BRANCH, and `./promote.sh main` fast forwards `main` to `dev`, each
# only when the source's tree carries the stamp `githooks/gate.sh` asks of
# the target: portable for `dev`, full for `main`. Without one, the gate runs
# in the clean worktree that has the source checked out. A target checked out
# somewhere is fast forwarded in that checkout, any other is moved by ref
# alone; no checkout is ever switched. Works on the repository of the current
# directory. `--push` then pushes the target to origin, through the hook.
set -uo pipefail

GATE="$(cd "$(dirname "$0")" && pwd)/githooks/gate.sh"
usage() { echo "usage: $0 [--push] BRANCH | main" >&2; exit 2; }
refuse() { echo "promote: $1" >&2; exit 1; }

PUSH=0
SOURCE=""
for arg in "$@"; do
  case "$arg" in
    --push) PUSH=1 ;;
    -*) usage ;;
    *) [[ -n $SOURCE ]] && usage; SOURCE=$arg ;;
  esac
done
[[ -z $SOURCE || $SOURCE == dev ]] && usage
if [[ $SOURCE == main ]]; then SOURCE=dev TARGET=main; else TARGET=dev; fi
LEVEL=$("$GATE" level "$TARGET")
FLAG=""
[[ $LEVEL == portable ]] && FLAG=--portable

checkout_of() {  # <branch>: the worktree that has it checked out, nothing if none
  git worktree list --porcelain \
    | awk -v want="branch refs/heads/$1" '/^worktree /{path = substr($0, 10)} $0 == want {print path}'
}

SOURCE_SHA=$(git rev-parse --verify -q "refs/heads/$SOURCE^{commit}") || refuse "no branch $SOURCE"
TARGET_SHA=$(git rev-parse --verify -q "refs/heads/$TARGET^{commit}") || refuse "no branch $TARGET"
git merge-base --is-ancestor "$TARGET_SHA" "$SOURCE_SHA" \
  || refuse "$SOURCE is not ahead of $TARGET, rebase it onto $TARGET first"

if ! WHY=$("$GATE" check "$SOURCE_SHA" "$LEVEL" 2>&1); then
  echo "promote: $WHY"
  GATE_CHECKOUT=$(checkout_of "$SOURCE")
  [[ -z $GATE_CHECKOUT ]] && refuse "$SOURCE is not checked out anywhere to run the gate in"
  if [[ -n $(git -C "$GATE_CHECKOUT" status --porcelain --untracked-files=normal) ]]; then
    refuse "the checkout of $SOURCE at $GATE_CHECKOUT is not clean"
  fi
  echo "promote: running ./verify.sh $FLAG in $GATE_CHECKOUT"
  (cd "$GATE_CHECKOUT" && ./verify.sh $FLAG) || refuse "the gate failed on $SOURCE"
  "$GATE" check "$SOURCE_SHA" "$LEVEL" || refuse "the gate passed but left no $LEVEL stamp on $SOURCE"
fi

TARGET_CHECKOUT=$(checkout_of "$TARGET")
if [[ -n $TARGET_CHECKOUT ]]; then
  git -C "$TARGET_CHECKOUT" merge --ff-only -q "$SOURCE_SHA" \
    || refuse "could not fast forward the checkout of $TARGET at $TARGET_CHECKOUT"
else
  git update-ref -m "promote: $SOURCE into $TARGET" "refs/heads/$TARGET" "$SOURCE_SHA" "$TARGET_SHA" \
    || refuse "$TARGET moved during the promotion"
fi
echo "promote: $TARGET ${TARGET_SHA:0:7} -> ${SOURCE_SHA:0:7}, $SOURCE passed the $LEVEL gate"

if [[ $PUSH -eq 1 ]]; then
  git push origin "refs/heads/$TARGET:refs/heads/$TARGET" || refuse "the push of $TARGET failed"
fi
