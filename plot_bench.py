#!/usr/bin/env python3
"""
Parse criterion benchmark output and generate an interactive N-scaling chart.

Usage:
    python3 plot_bench.py [target/criterion]   # default: target/criterion
    # Then open bench_report.html in a browser.
"""

import json
import sys
from pathlib import Path
from collections import defaultdict


def parse(criterion_dir: Path) -> dict:
    """
    Returns {sort_name: [(n, mean_ns, std_err_ns), ...]} sorted by n.
    Expects:  criterion_dir/sorts/n=<N>/<sort_name>/new/estimates.json
    """
    data = defaultdict(list)
    sorts_dir = criterion_dir / "sorts"

    if not sorts_dir.exists():
        print(f"No benchmark data found in {sorts_dir}", file=sys.stderr)
        return {}

    for n_dir in sorts_dir.iterdir():
        if not n_dir.is_dir() or not n_dir.name.startswith("n="):
            continue
        try:
            n = int(n_dir.name[2:])
        except ValueError:
            continue

        for sort_dir in n_dir.iterdir():
            if not sort_dir.is_dir():
                continue
            est_file = sort_dir / "new" / "estimates.json"
            if not est_file.exists():
                continue
            try:
                with open(est_file) as f:
                    est = json.load(f)
                data[sort_dir.name].append((
                    n,
                    est["mean"]["point_estimate"],
                    est["mean"]["standard_error"],
                ))
            except (KeyError, json.JSONDecodeError) as e:
                print(f"Warning: skipping {est_file}: {e}", file=sys.stderr)

    for name in data:
        data[name].sort(key=lambda t: t[0])

    return dict(data)


def generate(data: dict, output: Path):
    traces = []
    for name, points in sorted(data.items()):
        ns       = [p[0]          for p in points]
        means_us = [p[1] / 1_000  for p in points]   # ns → µs
        errs_us  = [p[2] / 1_000  for p in points]

        traces.append({
            "name": name,
            "x": ns,
            "y": means_us,
            "error_y": {"type": "data", "array": errs_us, "visible": True},
            "mode": "lines+markers",
            "type": "scatter",
        })

    traces_json = json.dumps(traces)

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Sort benchmark — N scaling</title>
<script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
<style>
  * {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{ font-family: sans-serif; background: #111; color: #eee;
          display: flex; flex-direction: column; height: 100vh; padding: 12px; gap: 10px; }}
  h1 {{ font-size: 1.1em; font-weight: 500; }}
  #controls {{ display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }}
  input  {{ background: #222; color: #eee; border: 1px solid #555;
             padding: 5px 10px; border-radius: 4px; font-size: 13px; width: 320px; }}
  button {{ background: #2a2a2a; color: #ddd; border: 1px solid #555;
             padding: 5px 12px; border-radius: 4px; cursor: pointer; font-size: 13px; }}
  button:hover {{ background: #3a3a3a; }}
  #count {{ font-size: 12px; color: #888; }}
  #chart {{ flex: 1; min-height: 0; }}
</style>
</head>
<body>
<h1>Sort benchmark — mean time (µs) vs N</h1>
<div id="controls">
  <input id="filter" type="text" placeholder="Filter by name (regex)…" oninput="applyFilter()">
  <button onclick="toggleLogY()">Toggle log Y</button>
  <button onclick="toggleLogX()">Toggle log X</button>
  <button onclick="clearFilter()">Clear</button>
  <span id="count"></span>
</div>
<div id="chart"></div>

<script>
const ALL = {traces_json};
let logY = false, logX = true;

const LAYOUT = () => ({{
  paper_bgcolor: '#111', plot_bgcolor: '#1a1a1a',
  font: {{ color: '#ddd', size: 12 }},
  xaxis: {{ title: 'N (array size)',   type: logX ? 'log' : 'linear', gridcolor: '#2e2e2e', color: '#aaa' }},
  yaxis: {{ title: 'mean time (µs)',   type: logY ? 'log' : 'linear', gridcolor: '#2e2e2e', color: '#aaa' }},
  legend: {{ bgcolor: '#1a1a1a', bordercolor: '#444', borderwidth: 1,
             font: {{ size: 11 }}, itemclick: 'toggle', itemdoubleclick: 'toggleothers' }},
  margin: {{ l: 70, r: 20, t: 10, b: 55 }},
  hovermode: 'closest',
}});

const CONFIG = {{ responsive: true }};

Plotly.newPlot('chart', ALL.map(t => ({{...t, visible: true}})), LAYOUT(), CONFIG);
updateCount();

function regex() {{
  const v = document.getElementById('filter').value.trim();
  if (!v) return null;
  try {{ return new RegExp(v, 'i'); }} catch {{ return null; }}
}}

function applyFilter() {{
  const re = regex();
  Plotly.restyle('chart', {{ visible: ALL.map(t => (!re || re.test(t.name)) ? true : 'legendonly') }});
  updateCount();
}}

function clearFilter() {{
  document.getElementById('filter').value = '';
  applyFilter();
}}

function toggleLogY() {{
  logY = !logY;
  Plotly.relayout('chart', {{ 'yaxis.type': logY ? 'log' : 'linear' }});
}}

function toggleLogX() {{
  logX = !logX;
  Plotly.relayout('chart', {{ 'xaxis.type': logX ? 'log' : 'linear' }});
}}

function updateCount() {{
  const re = regex();
  const n = ALL.filter(t => !re || re.test(t.name)).length;
  document.getElementById('count').textContent = n + '\u202f/\u202f' + ALL.length + ' sorts';
}}
</script>
</body>
</html>"""

    output.write_text(html)
    print(f"Report written → {output}")


def main():
    criterion_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("target/criterion")
    data = parse(criterion_dir)

    if not data:
        print("No data found. Run `cargo bench --bench sorts` first.")
        sys.exit(1)

    total = sum(len(v) for v in data.values())
    print(f"Loaded {len(data)} sorts, {total} data points.")
    generate(data, Path("bench_report.html"))


if __name__ == "__main__":
    main()
