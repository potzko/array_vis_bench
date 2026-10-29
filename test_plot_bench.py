# Unit tests for plot_bench: an archive's header, points and entry lines,
# the archives joined into the page's data, and the page itself.
import json
import re
import unittest

from plot_bench import load, page, read_archive

SMALL = (
    "# n=100 k=3 input=shuffled\n"
    "quick sort<part: hoare>\t1000\t10\t0\n"
    "quick sort<part: lomuto>\t1500\t10\t0\n"
    "heap sort\t2000\t10\t1\n"
    "# entry\tquick sort<part: hoare>\tsort\tPart\thoare\tpart\n"
    "# entry\tquick sort<part: lomuto>\tsort\tPart\tlomuto\tpart\n"
    "# entry\theap sort\tsort\n"
)
LARGE = "# n=1000 k=3\nquick sort<part: hoare>\t20000\nheap sort\t45000\n"
SWEEP = (
    "# sweep budget_ms=600 spent_ms=610 share=5 limit_ns=1000 input=nearly sorted max_size=4194304\n"
    "heap sort\t1\t10\nheap sort\t2\t25\n"
    "# fits\theap sort\t2\n"
    "# entry\theap sort\tsort\n"
)


class ReadArchiveTest(unittest.TestCase):
    def test_a_timing_archive_gives_each_entry_one_point_at_its_size_on_its_input(self) -> None:
        archive = read_archive(SMALL)
        self.assertEqual(archive.input, "shuffled")
        self.assertEqual(archive.points, [("quick sort<part: hoare>", 100, 1000), ("quick sort<part: lomuto>", 100, 1500), ("heap sort", 100, 2000)])

    def test_the_entry_lines_give_the_kind_and_the_axes(self) -> None:
        entries = read_archive(SMALL).entries
        self.assertEqual(entries["quick sort<part: hoare>"], ("sort", [("Part", "hoare", "part")]))
        self.assertEqual(entries["heap sort"], ("sort", []))

    def test_an_archive_without_an_input_ran_on_the_primary_one(self) -> None:
        self.assertEqual(read_archive(LARGE).input, "primary")

    def test_a_sweep_header_value_may_hold_spaces(self) -> None:
        archive = read_archive(SWEEP)
        self.assertEqual(archive.input, "nearly sorted")
        self.assertEqual(archive.points, [("heap sort", 1, 10), ("heap sort", 2, 25)])

    def test_an_archive_without_a_header_is_refused(self) -> None:
        with self.assertRaises(ValueError): read_archive("a\t1\n")


class LoadTest(unittest.TestCase):
    def test_entries_are_listed_once_with_their_family_and_series_per_input(self) -> None:
        data = load([LARGE, SMALL, SWEEP])
        names = [entry["name"] for entry in data["entries"]]
        self.assertEqual(names, ["heap sort", "quick sort<part: hoare>", "quick sort<part: lomuto>"])
        self.assertEqual(data["entries"][1], {"name": "quick sort<part: hoare>", "kind": "sort", "family": "quick sort", "axes": [["Part", "hoare", "part"]]})
        self.assertEqual(data["inputs"], ["nearly sorted", "primary", "shuffled"])
        self.assertEqual(data["sizes"], [1, 2, 100, 1000])
        # A series is the entry's index, its input, then sizes and microseconds.
        self.assertIn([0, "nearly sorted", [1, 2], [0.01, 0.025]], data["series"])
        self.assertIn([1, "primary", [1000], [20.0]], data["series"])

    def test_an_entry_no_archive_describes_is_kind_unknown_with_no_axes(self) -> None:
        data = load([LARGE])
        self.assertEqual(data["entries"][0], {"name": "heap sort", "kind": "unknown", "family": "heap sort", "axes": []})

    def test_the_same_entry_input_and_size_twice_is_refused(self) -> None:
        with self.assertRaises(ValueError): load([SMALL, SMALL])
        with self.assertRaises(ValueError): load([SWEEP, SWEEP])

    def test_a_timing_point_replaces_a_sweep_point_at_its_size_and_input(self) -> None:
        timing = "# n=2 k=15 input=nearly sorted\nheap sort\t20\t1\t0\n"
        for archives in ([SWEEP, timing], [timing, SWEEP]):
            self.assertEqual(load(archives)["series"], [[0, "nearly sorted", [1, 2], [0.01, 0.02]]])

    def test_times_keep_four_significant_digits(self) -> None:
        data = load(["# n=10 k=1\nx\t123456789\n"])
        self.assertEqual(data["series"][0][3], [123500.0])


class PageTest(unittest.TestCase):
    def test_the_page_inlines_the_data_and_both_scripts(self) -> None:
        html = page(load([SMALL]))
        found = re.search(r"const DATA = (.*?);</script>", html)
        assert found is not None
        self.assertEqual(json.loads(found[1])["inputs"], ["shuffled"])
        self.assertIn("function facetValues", html)
        self.assertIn("Plotly.react", html)
        self.assertNotIn("__", html.split("<script>")[1])

    def test_a_closing_script_tag_in_a_name_cannot_end_the_data(self) -> None:
        html = page(load(["# n=10 k=1\n</script>\t5\n"]))
        self.assertNotIn("</script>\\t", html)
        self.assertEqual(html.count("</script>"), html.count("<script"))


if __name__ == "__main__":
    unittest.main()
