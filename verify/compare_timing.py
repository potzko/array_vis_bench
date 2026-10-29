# Compare a timing archive against its baseline and fail on a regression.
# `regressions` is the pure comparison over two name to nanoseconds maps;
# `main` reads the two TSV files `--timing --out` writes, skipping the `# n=
# k=` header that names the setup, and prints a report.
# A single entry regresses when it is slower by more than the entry
# tolerance and by more than the floor, which keeps microsecond entries out
# of the gate; the whole catalog regresses when the sum of the
# shared medians grows by more than the total tolerance and the same floor.
import argparse
import sys
from pathlib import Path
from typing import NamedTuple


class Report(NamedTuple):
    total_ratio: float
    slow_entries: list[tuple[str, int, int]]
    added: list[str]
    removed: list[str]
    total_failed: bool

    @property
    def failed(self) -> bool:
        return self.total_failed or bool(self.slow_entries)


def read_archive(path: Path) -> dict[str, int]:
    return parse_archive(path.read_text())


# The first two columns, the name and the median; a newer archive adds the
# spread and the outlier count after them.
def parse_archive(text: str) -> dict[str, int]:
    rows = (line.split("\t") for line in text.splitlines() if line.strip() and not line.startswith("#"))
    return {row[0]: int(row[1]) for row in rows}


def regressions(
    baseline: dict[str, int],
    current: dict[str, int],
    entry_tolerance: float = 0.3,
    entry_floor_ns: int = 50_000,
    total_tolerance: float = 0.10,
) -> Report:
    shared = sorted(baseline.keys() & current.keys())
    base_total = sum(baseline[name] for name in shared)
    now_total = sum(current[name] for name in shared)
    total_ratio = now_total / base_total if base_total else 1.0
    slow = [
        (name, baseline[name], current[name])
        for name in shared
        if current[name] > baseline[name] * (1 + entry_tolerance)
        and current[name] - baseline[name] > entry_floor_ns
    ]
    slow.sort(key=lambda row: row[2] / row[1], reverse=True)
    return Report(
        total_ratio=total_ratio,
        slow_entries=slow,
        added=sorted(current.keys() - baseline.keys()),
        removed=sorted(baseline.keys() - current.keys()),
        total_failed=total_ratio > 1 + total_tolerance and now_total - base_total > entry_floor_ns,
    )


def render(report: Report, total_tolerance: float) -> str:
    lines = [f"total of shared medians: {report.total_ratio:.3f}x the baseline (limit {1 + total_tolerance:.2f}x)"]
    lines += [f"  SLOW {name}: {base / 1000:.1f}us -> {now / 1000:.1f}us ({now / base:.2f}x)" for name, base, now in report.slow_entries]
    if report.added: lines.append(f"  {len(report.added)} entries not in the baseline, first: {report.added[0]}")
    if report.removed: lines.append(f"  {len(report.removed)} baseline entries missing, first: {report.removed[0]}")
    lines.append("FAIL: timing regressed" if report.failed else "OK: no timing regression")
    return "\n".join(lines)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="fail when a timing archive regressed against its baseline")
    parser.add_argument("baseline", type=Path)
    parser.add_argument("current", type=Path)
    parser.add_argument("--entry-tolerance", type=float, default=0.3, help="allowed slowdown of one entry, as a fraction")
    parser.add_argument("--entry-floor-ns", type=int, default=50_000, help="an entry slower by less than this never fails")
    parser.add_argument("--total-tolerance", type=float, default=0.10, help="allowed growth of the summed medians")
    args = parser.parse_args(argv)
    report = regressions(
        read_archive(args.baseline), read_archive(args.current),
        args.entry_tolerance, args.entry_floor_ns, args.total_tolerance,
    )
    print(render(report, args.total_tolerance))
    return 1 if report.failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
