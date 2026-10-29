#!/usr/bin/env python3
# Write the benchmark page from timing and sweep archives. `read_archive`
# reads one archive's input shape, points and entry lines, `load` joins the
# archives into the page's data, every entry once with its kind, family and
# axes and one series per entry and input, and `page` inlines that data with
# the scripts in `bench_view/` into one self contained HTML file.
#
#   for input in shuffled "nearly sorted" descending; do
#     ./target/release/array_vis_bench --timing 1000 15 --input "$input" --out "target/timing_$input.tsv"
#   done
#   ./target/release/array_vis_bench --sweep 600 --out target/sweep.tsv
#   python3 plot_bench.py target/timing_*.tsv target/sweep.tsv --out target/bench_report.html
import argparse
import json
import re
import sys
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path

VIEW = Path(__file__).resolve().parent / "bench_view"

# A header field runs to the next ` key=`, so a value such as an input shape
# may hold spaces.
FIELD = re.compile(r"(\w+)=(.*?)(?= \w+=|$)")


@dataclass
class Archive:
    sweep: bool
    input: str
    points: list[tuple[str, int, int]]
    entries: dict[str, tuple[str, list[tuple[str, str, str]]]]


def read_archive(text: str) -> Archive:
    lines = text.splitlines()
    if not lines or not (lines[0].startswith("# sweep ") or lines[0].startswith("# n=")):
        raise ValueError("the archive has no `# n=` or `# sweep` header, it was not written by --timing or --sweep --out")
    header = dict(FIELD.findall(lines[0].removeprefix("# sweep ").removeprefix("# ")))
    rows = [line.split("\t") for line in lines[1:] if line.strip() and not line.startswith("#")]
    if lines[0].startswith("# sweep"): points = [(row[0], int(row[1]), int(row[2])) for row in rows]
    else: points = [(row[0], int(header["n"]), int(row[1])) for row in rows]
    described = (line.split("\t")[1:] for line in lines if line.startswith("# entry\t"))
    entries = {fields[0]: (fields[1], list(zip(fields[2::3], fields[3::3], fields[4::3]))) for fields in described}
    return Archive(lines[0].startswith("# sweep"), header.get("input", "primary"), points, entries)


def family_of(name: str) -> str:
    return name.split("<", 1)[0].strip()


def micros(ns: int) -> float:
    return float(f"{ns / 1000:.4g}")


# The page's data: every entry once, by name, with its kind, family and axes;
# the inputs and sizes met; and one series per entry and input as the entry's
# index, the input, the sizes ascending and the times in microseconds. A
# timing archive's median over its rounds replaces a sweep's single batch at
# the same entry, input and size; two archives of one kind there clash.
def load(texts: Iterable[str]) -> dict:
    archives = [read_archive(text) for text in texts]
    described = {name: entry for archive in archives for name, entry in archive.entries.items()}
    held: dict[tuple[str, str], dict[int, tuple[int, bool]]] = {}
    for archive in archives:
        for name, n, ns in archive.points:
            points = held.setdefault((name, archive.input), {})
            if n in points and points[n][1] == archive.sweep:
                raise ValueError(f"`{name}` on {archive.input} at n={n} twice, keep one archive of each kind per size and input")
            if n not in points or points[n][1]: points[n] = (ns, archive.sweep)
    series = {key: {n: ns for n, (ns, _) in points.items()} for key, points in held.items()}
    names = sorted({name for name, _ in series})
    index = {name: i for i, name in enumerate(names)}
    entries = [
        {"name": name, "kind": described.get(name, ("unknown", []))[0], "family": family_of(name),
         "axes": [list(axis) for axis in described.get(name, ("unknown", []))[1]]}
        for name in names
    ]
    return {
        "entries": entries,
        "inputs": sorted({input for _, input in series}),
        "sizes": sorted({n for points in series.values() for n in points}),
        "series": [[index[name], input, sorted(points), [micros(points[n]) for n in sorted(points)]] for (name, input), points in sorted(series.items())],
    }


# The scripts go in before the data, so no name in the data is read as a
# placeholder, and a `</` in the data, which would end its script element
# early, is escaped the way JSON allows.
def page(data: dict) -> str:
    template = (VIEW / "page.html").read_text()
    script = json.dumps(data, separators=(",", ":")).replace("</", "<\\/")
    return (
        template.replace("/*__VIEW__*/", (VIEW / "view.js").read_text())
        .replace("/*__PAGE__*/", (VIEW / "page.js").read_text())
        .replace("__DATA__", script)
    )


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="write the benchmark page from timing and sweep archives")
    parser.add_argument("archives", type=Path, nargs="+", help="archives written by `--timing N K --out` or `--sweep SECONDS --out`")
    parser.add_argument("--out", type=Path, default=Path("target/bench_report.html"))
    args = parser.parse_args(argv)
    data = load(path.read_text() for path in args.archives)
    args.out.write_text(page(data))
    print(f"{len(data['entries'])} entries on {', '.join(data['inputs'])}, {len(data['series'])} series, report at {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
