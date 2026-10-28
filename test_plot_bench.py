# Unit tests for plot_bench: the setup header, the series joined across
# archives, the selector tree by type head and the trace shapes.
import unittest

from plot_bench import load, read_setup, traces, tree

SMALL = "# n=100 k=3\nquick sort<part: hoare>\t1000\nquick sort<part: lomuto>\t1500\nheap sort\t2000\n"
LARGE = "# n=1000 k=3\nquick sort<part: hoare>\t20000\nheap sort\t45000\n"


class ReadSetupTest(unittest.TestCase):
    def test_the_header_names_the_input_size_and_run_count(self) -> None:
        self.assertEqual(read_setup(SMALL), (100, 3))

    def test_an_archive_without_a_header_is_refused(self) -> None:
        with self.assertRaises(ValueError): read_setup("a\t1\n")


class LoadTest(unittest.TestCase):
    def test_each_entry_gets_one_point_per_archive_sorted_by_size(self) -> None:
        series = load([LARGE, SMALL])
        self.assertEqual(series["quick sort<part: hoare>"], [(100, 1000), (1000, 20000)])
        self.assertEqual(series["quick sort<part: lomuto>"], [(100, 1500)])

    def test_two_archives_at_one_size_are_refused(self) -> None:
        with self.assertRaises(ValueError): load([SMALL, SMALL])


class TreeTest(unittest.TestCase):
    def test_entries_group_under_their_type_head(self) -> None:
        root = tree(["quick sort<part: hoare>", "quick sort<part: lomuto>", "heap sort"])
        self.assertEqual([branch["label"] for branch in root["children"]], ["heap sort", "quick sort"])
        quick = root["children"][1]
        self.assertEqual([leaf["name"] for leaf in quick["children"]], ["quick sort<part: hoare>", "quick sort<part: lomuto>"])
        self.assertEqual(quick["children"][0]["label"], "part: hoare")
        self.assertEqual(root["children"][0]["children"][0]["label"], "heap sort")


class TracesTest(unittest.TestCase):
    def test_a_trace_plots_microseconds_against_size(self) -> None:
        [trace] = traces({"heap sort": [(100, 2000), (1000, 45000)]})
        self.assertEqual((trace["name"], trace["x"], trace["y"]), ("heap sort", [100, 1000], [2.0, 45.0]))


if __name__ == "__main__":
    unittest.main()
