# `githooks`

The gate between the work branches and the two shared ones. `dev` takes a
tree that passed `../verify.sh --portable`, the build, the catalog, the script
tests and the workspace tests, none of which depend on the machine. `main`
takes a tree that passed the full `../verify.sh`, which adds the assembly,
the `compare_sorts` gate and the timing comparison and so passes only on the
machine the baselines in `../verify/` were taken on.

- `gate.sh`: the policy. Which level each branch needs, and the stamps. A run
  of `verify.sh` that passes without `--bless` or `--quick`, in a checkout
  that was clean when it started and still is when it ends, writes a stamp
  named after the tree's sha under `verify-stamps/` in the common git dir.
  The stamp holds the level, `portable` or `full`, and the `rustc -V` of the
  run; a full stamp is never replaced by a portable one, and the check for
  `main` refuses a full stamp taken under another toolchain. Every worktree
  of the clone shares the stamps, and a rebase that leaves the tree as it was
  keeps its stamp.
- `pre-push`: refuses a push to `dev` or `main` on any remote unless the
  pushed tip's tree carries the stamp that branch needs. Deleting either is
  not gated, and other branches are not gated at all. A push run as
  `GATE_BYPASS="reason" git push ...` goes through anyway and appends the
  reason to `gate-bypass.log` in the common git dir; `git push --no-verify`
  skips the hook without a trace.
- `test_gate.py`: the tests, which run real git in scratch repositories.

The hooks take effect once the clone points at this folder, by absolute path
so that every worktree runs the copy in the main checkout rather than its
own: `git config core.hooksPath "$PWD/githooks"` from the main checkout. The
setting is per clone, so a new clone has no gate until it is set.

After a `--bless`, commit the new baselines and run the gate again: a bless
never stamps, since it compared nothing.
