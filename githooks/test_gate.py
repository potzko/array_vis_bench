# Tests for the branch gate: the stamps `gate.sh` writes and checks. Each test
# drives real git in a scratch repository with its own identity and default
# branch and with the global and system config shut out, so the settings of
# whoever runs the tests cannot change a result.
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

HOOKS = Path(__file__).resolve().parent
GATE = HOOKS / "gate.sh"
ENV = {name: value for name, value in os.environ.items() if not name.startswith(("GIT_", "GATE_"))} | {
    "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"}
RUSTC = subprocess.run(["rustc", "-V"], capture_output=True, text=True, check=True).stdout.strip()


def run(args: list, cwd: Path, env: dict | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(
        [str(arg) for arg in args], cwd=cwd, env=ENV | (env or {}), capture_output=True, text=True, check=False)


class Repo:
    # A scratch repository on `dev` with one commit and `main` beside it.
    def __init__(self, root: Path) -> None:
        self.root, self.path = root, root / "repo"
        run(["git", "init", "-q", "-b", "dev", self.path], root)
        self.git("config", "user.name", "gate test")
        self.git("config", "user.email", "gate@test")
        self.commit("a")
        self.git("branch", "main")

    def git(self, *args: str, cwd: Path | None = None) -> str:
        done = run(["git", *args], cwd or self.path)
        assert done.returncode == 0, f"git {' '.join(args)}: {done.stderr}"
        return done.stdout.strip()

    def commit(self, name: str, cwd: Path | None = None) -> str:
        (cwd or self.path).joinpath(name).write_text(name)
        self.git("add", name, cwd=cwd)
        self.git("commit", "-q", "-m", name, cwd=cwd)
        return self.git("rev-parse", "HEAD", cwd=cwd)

    def gate(self, *args: str, cwd: Path | None = None) -> subprocess.CompletedProcess:
        return run([GATE, *args], cwd or self.path)

    def stamp(self, level: str, cwd: Path | None = None) -> subprocess.CompletedProcess:
        return self.gate("stamp", level, self.gate("clean-tree", cwd=cwd).stdout.strip(), cwd=cwd)

    def stamp_file(self, rev: str = "HEAD") -> Path:
        common = Path(self.git("rev-parse", "--path-format=absolute", "--git-common-dir"))
        return common / "verify-stamps" / self.git("rev-parse", f"{rev}^{{tree}}")


def scratch_repo(test: unittest.TestCase) -> Repo:
    root = tempfile.TemporaryDirectory()
    test.addCleanup(root.cleanup)
    return Repo(Path(root.name))


class StampTest(unittest.TestCase):
    def setUp(self) -> None:
        self.repo = scratch_repo(self)

    def test_a_clean_run_stamps_its_tree_with_the_level_and_the_toolchain(self) -> None:
        self.assertEqual(self.repo.stamp("portable").returncode, 0)
        self.assertEqual(self.repo.stamp_file().read_text().splitlines(), ["portable", RUSTC])

    def test_a_tree_dirty_at_the_start_gets_no_stamp(self) -> None:
        (self.repo.path / "a").write_text("edited")
        self.assertIn("not clean", self.repo.stamp("full").stdout)
        self.assertFalse(self.repo.stamp_file().exists())

    def test_an_untracked_file_makes_the_tree_dirty(self) -> None:
        (self.repo.path / "loose").write_text("loose")
        self.assertEqual(self.repo.gate("clean-tree").stdout, "")

    def test_a_tree_that_changed_during_the_run_gets_no_stamp(self) -> None:
        start = self.repo.gate("clean-tree").stdout.strip()
        self.repo.commit("b")
        self.assertIn("changed", self.repo.gate("stamp", "full", start).stdout)
        self.assertFalse(self.repo.stamp_file("HEAD~1").exists() or self.repo.stamp_file().exists())

    def test_a_portable_run_never_downgrades_a_full_stamp(self) -> None:
        self.repo.stamp("full")
        self.repo.stamp("portable")
        self.assertEqual(self.repo.stamp_file().read_text().splitlines()[0], "full")

    def test_a_full_run_replaces_a_portable_stamp(self) -> None:
        self.repo.stamp("portable")
        self.repo.stamp("full")
        self.assertEqual(self.repo.stamp_file().read_text().splitlines()[0], "full")

    def test_an_unknown_level_is_a_usage_error(self) -> None:
        self.assertEqual(self.repo.stamp("quick").returncode, 2)

    def test_every_worktree_shares_the_stamps(self) -> None:
        tree = self.repo.root / "wt"
        self.repo.git("worktree", "add", "-q", "-b", "feature", str(tree))
        self.repo.stamp("portable", cwd=tree)
        self.assertEqual(self.repo.gate("check", "feature", "portable").returncode, 0)


class CheckTest(unittest.TestCase):
    def setUp(self) -> None:
        self.repo = scratch_repo(self)

    def test_a_tree_without_a_stamp_fails_and_names_the_command(self) -> None:
        done = self.repo.gate("check", "HEAD", "portable")
        self.assertEqual(done.returncode, 1)
        self.assertIn("./verify.sh --portable", done.stderr)

    def test_a_portable_stamp_passes_portable_but_not_full(self) -> None:
        self.repo.stamp("portable")
        self.assertEqual(self.repo.gate("check", "HEAD", "portable").returncode, 0)
        self.assertEqual(self.repo.gate("check", "HEAD", "full").returncode, 1)

    def test_a_full_stamp_passes_both_levels(self) -> None:
        self.repo.stamp("full")
        self.assertEqual([self.repo.gate("check", "HEAD", level).returncode for level in ("portable", "full")], [0, 0])

    def test_a_commit_with_the_same_tree_shares_the_stamp(self) -> None:
        self.repo.stamp("portable")
        self.repo.git("commit", "-q", "--allow-empty", "-m", "same tree")
        self.assertEqual(self.repo.gate("check", "HEAD", "portable").returncode, 0)

    def test_a_full_stamp_from_another_toolchain_fails_only_the_full_check(self) -> None:
        self.repo.stamp("full")
        self.repo.stamp_file().write_text("full\nrustc 0.0.0\n")
        done = self.repo.gate("check", "HEAD", "full")
        self.assertEqual(done.returncode, 1)
        self.assertIn("rustc 0.0.0", done.stderr)
        self.assertEqual(self.repo.gate("check", "HEAD", "portable").returncode, 0)

    def test_dev_needs_portable_main_needs_full_and_the_rest_nothing(self) -> None:
        levels = [self.repo.gate("level", branch).stdout.strip() for branch in ("dev", "main", "feature")]
        self.assertEqual(levels, ["portable", "full", ""])


if __name__ == "__main__":
    unittest.main()
