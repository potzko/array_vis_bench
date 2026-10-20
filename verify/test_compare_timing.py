# Unit tests for compare_timing.regressions: the entry gate, its floor, the
# total gate and the bookkeeping of added and removed entries.
import unittest

from compare_timing import regressions


class RegressionsTest(unittest.TestCase):
    def test_identical_archives_pass(self) -> None:
        archive = {"a": 1_000_000, "b": 2_000_000}
        report = regressions(archive, dict(archive))
        self.assertFalse(report.failed)
        self.assertAlmostEqual(report.total_ratio, 1.0)

    def test_one_entry_past_tolerance_and_floor_fails(self) -> None:
        report = regressions({"a": 1_000_000, "b": 1_000_000}, {"a": 1_000_000, "b": 1_700_000})
        self.assertEqual([row[0] for row in report.slow_entries], ["b"])
        self.assertTrue(report.failed)

    def test_a_tiny_entry_below_the_floor_never_fails(self) -> None:
        report = regressions({"a": 1_000}, {"a": 5_000})
        self.assertEqual(report.slow_entries, [])
        self.assertFalse(report.failed)

    def test_the_summed_medians_have_their_own_gate(self) -> None:
        baseline = {name: 1_000_000 for name in "abcd"}
        current = {name: 1_200_000 for name in "abcd"}
        report = regressions(baseline, current)
        self.assertEqual(report.slow_entries, [])
        self.assertTrue(report.total_failed)

    def test_the_total_gate_has_the_same_floor(self) -> None:
        report = regressions({"a": 10_000, "b": 10_000}, {"a": 50_000, "b": 50_000})
        self.assertFalse(report.failed)

    def test_added_and_removed_entries_are_listed_not_failed(self) -> None:
        report = regressions({"a": 1, "gone": 1}, {"a": 1, "new": 1})
        self.assertEqual((report.added, report.removed), (["new"], ["gone"]))
        self.assertFalse(report.failed)


if __name__ == "__main__":
    unittest.main()
