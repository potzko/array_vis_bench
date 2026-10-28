#!/usr/bin/env python3
# Chart how every entry scales with the input size from timing archives, one
# archive per size. `read_setup` reads the `# n= k=` header an archive carries,
# `load` joins the archives into one series per entry, `tree` groups the
# entries by type head for the selector, `traces` shapes the series for the
# chart and `page` renders a self contained Plotly page.
#
#   for n in 100 1000 10000; do
#     ./target/release/array_vis_bench --timing $n 15 --out target/timing_$n.tsv
#   done
#   python3 plot_bench.py target/timing_*.tsv --out target/bench_report.html
import argparse
import json
import re
import sys
from collections import defaultdict
from collections.abc import Iterable
from itertools import groupby
from pathlib import Path

from verify.compare_timing import parse_archive

SETUP = re.compile(r"^# n=(\d+) k=(\d+)$", re.MULTILINE)


def read_setup(text: str) -> tuple[int, int]:
    found = SETUP.search(text)
    if found is None: raise ValueError("the archive has no `# n= k=` header, it was not written by --timing --out")
    return int(found[1]), int(found[2])


# name -> [(n, median nanoseconds)] ascending in n, one point per archive.
def load(texts: Iterable[str]) -> dict[str, list[tuple[int, int]]]:
    series: dict[str, list[tuple[int, int]]] = defaultdict(list)
    seen: set[int] = set()
    for text in texts:
        n, _ = read_setup(text)
        if n in seen: raise ValueError(f"two archives at n={n}, keep one per size")
        seen.add(n)
        for name, ns in parse_archive(text).items(): series[name].append((n, ns))
    return {name: sorted(points) for name, points in series.items()}


def head_of(name: str) -> str:
    return name.split("<", 1)[0].strip()


# The selector tree: one branch per type head, one leaf per entry, labelled
# by what follows the head so the branch label is not repeated down the list.
def tree(names: Iterable[str]) -> dict:
    def leaf(name: str) -> dict:
        rest = name[len(head_of(name)):].strip("<> ")
        return {"label": rest or name, "is_leaf": True, "name": name}
    branches = [
        {"label": head, "is_leaf": False, "children": [leaf(n) for n in members]}
        for head, members in groupby(sorted(names), key=head_of)
    ]
    return {"label": "all", "is_leaf": False, "children": branches}


def traces(series: dict[str, list[tuple[int, int]]]) -> list[dict]:
    return [
        {
            "name": name,
            "x": [n for n, _ in points],
            "y": [ns / 1000 for _, ns in points],
            "mode": "lines+markers",
            "line": {"width": 2},
            "marker": {"size": 6},
            "hovertemplate": "%{fullData.name}<br>n=%{x}: %{y:.1f} us<extra></extra>",
        }
        for name, points in sorted(series.items())
    ]


def page(chart: list[dict], selector: dict, sizes: list[int]) -> str:
    return TEMPLATE.replace("__TRACES__", json.dumps(chart)).replace("__TREE__", json.dumps(selector)).replace("__NS__", json.dumps(sizes))


# The page is one chart with a selector tree beside it. Branches default on
# and leaves off, so a leaf shows when it and every ancestor are on; the
# per branch all and none buttons bulk toggle a subtree. The slider picks a
# size that ranks the legend and drives the faster or slower than filter.
TEMPLATE = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Scaling with input size</title>
<script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body { font-family: sans-serif; background: #111; color: #eee;
         display: flex; height: 100vh; padding: 10px; gap: 10px; }
  #sidebar { width: 360px; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  #sidebar h2 { font-size: 1em; font-weight: 500; color: #bbb; border-bottom: 1px solid #333; padding-bottom: 4px; }
  #sidebar-buttons { display: flex; gap: 6px; flex-wrap: wrap; }
  #tree { flex: 1; min-height: 0; overflow: auto; background: #1a1a1a; border: 1px solid #2e2e2e;
          border-radius: 4px; padding: 6px 8px; font-size: 12px; }
  details > summary { list-style: none; cursor: pointer; display: flex; align-items: center; gap: 4px;
                      padding: 1px 0; user-select: none; }
  details > summary::-webkit-details-marker { display: none; }
  details > summary::before { content: ">"; font-size: 9px; color: #666; width: 10px; flex-shrink: 0; transition: transform 0.1s; }
  details[open] > summary::before { transform: rotate(90deg); }
  .tree-leaf { display: flex; align-items: center; gap: 4px; padding: 1px 0 1px 14px; }
  .tree-branch-children { margin-left: 12px; border-left: 1px solid #2a2a2a; padding-left: 4px; }
  .tree-label { flex: 1; cursor: pointer; color: #ccc; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .tree-label.leaf { color: #888; font-size: 11px; }
  input[type=checkbox] { flex-shrink: 0; accent-color: #4a9eff; cursor: pointer; }
  .subtree-btn { font-size: 10px; padding: 0 5px; height: 16px; line-height: 14px; background: #2a2a2a;
                 border: 1px solid #444; border-radius: 2px; color: #aaa; cursor: pointer; flex-shrink: 0; }
  .subtree-btn:hover { background: #3a3a3a; color: #fff; }
  #main { flex: 1; display: flex; flex-direction: column; gap: 8px; min-width: 0; min-height: 0; }
  h1 { font-size: 1.05em; font-weight: 500; }
  .controls { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; background: #1a1a1a;
              border: 1px solid #2e2e2e; border-radius: 4px; padding: 8px 10px; font-size: 12px; }
  .controls input[type=text] { background: #222; color: #eee; border: 1px solid #555; padding: 4px 8px;
                               border-radius: 3px; font-size: 12px; width: 280px; }
  .controls input[type=range] { width: 200px; }
  button { background: #2a2a2a; color: #ddd; border: 1px solid #555; padding: 4px 10px; border-radius: 3px;
           cursor: pointer; font-size: 12px; }
  button:hover { background: #3a3a3a; }
  .ctl-label { color: #888; }
  #count { color: #888; margin-left: auto; }
  .ctl-readout { color: #4a9eff; font-variant-numeric: tabular-nums; max-width: 380px; overflow: hidden;
                 text-overflow: ellipsis; white-space: nowrap; }
  select { background: #222; color: #eee; border: 1px solid #555; padding: 4px 8px; border-radius: 3px; font-size: 12px; }
  #ref-input { width: 320px; }
  #chart { flex: 1; min-height: 0; }
</style>
</head>
<body>
<div id="sidebar">
  <h2>Entries</h2>
  <div id="sidebar-buttons">
    <button onclick="setAllBranches(true)">Branches on</button>
    <button onclick="setAllBranches(false)">Branches off</button>
    <button onclick="setAllLeaves(true)">Leaves on</button>
    <button onclick="setAllLeaves(false)">Leaves off</button>
  </div>
  <div id="tree"></div>
</div>
<div id="main">
  <h1>Median time (us) against input size n</h1>
  <div class="controls">
    <input id="filter" type="text" placeholder="Regex filter" oninput="applyVisibility()">
    <button onclick="document.getElementById('filter').value=''; applyVisibility()">Clear filter</button>
    <span class="ctl-label">n =</span>
    <input id="size-slider" type="range" min="-1" step="1" oninput="onSliderInput()" onchange="applyVisibility()">
    <span id="size-readout" class="ctl-readout">off</span>
    <button onclick="toggleLog('yaxis')">log y</button>
    <button onclick="toggleLog('xaxis')">log x</button>
    <span id="count"></span>
  </div>
  <div class="controls">
    <span class="ctl-label">Show entries</span>
    <select id="perf-mode" onchange="applyVisibility()">
      <option value="off" selected>all</option>
      <option value="faster">faster than</option>
      <option value="slower">slower than</option>
    </select>
    <input id="ref-input" list="ref-options" type="text" placeholder="reference entry, type or click a trace" oninput="onRefInputChange()">
    <button onclick="setRef(null)">Clear</button>
    <span class="ctl-label">at the selected n. Reference:</span>
    <span id="ref-readout" class="ctl-readout">none</span>
  </div>
  <div id="chart"></div>
</div>
<datalist id="ref-options"></datalist>
<script>
const ALL = __TRACES__;
const TREE = __TREE__;
const NS = __NS__;
const nameToTrace = Object.create(null);
ALL.forEach(t => nameToTrace[t.name] = t);
const datalist = document.getElementById('ref-options');
for (const t of ALL) { const o = document.createElement('option'); o.value = t.name; datalist.appendChild(o); }

const treeState = Object.create(null), parentOf = Object.create(null), nameToId = Object.create(null);
const checkboxById = Object.create(null);
let nextId = 0;
function walkTree(node, parentId) {
  const id = nextId++;
  parentOf[id] = parentId;
  treeState[id] = !node.is_leaf;
  if (node.is_leaf) nameToId[node.name] = id;
  for (const c of (node.children || [])) walkTree(c, id);
  node.id = id;
}
walkTree(TREE, -1);
function leafVisible(name) {
  let id = nameToId[name];
  if (id === undefined) return true;
  for (; id !== -1; id = parentOf[id]) if (!treeState[id]) return false;
  return true;
}
function setState(id, value) { treeState[id] = value; if (checkboxById[id]) checkboxById[id].checked = value; }
function checkbox(id) {
  const cb = document.createElement('input');
  cb.type = 'checkbox';
  cb.checked = treeState[id];
  cb.addEventListener('click', e => e.stopPropagation());
  cb.addEventListener('change', () => { treeState[id] = cb.checked; applyVisibility(); });
  checkboxById[id] = cb;
  return cb;
}
function subtreeButton(node, text, value) {
  const btn = document.createElement('button');
  btn.className = 'subtree-btn';
  btn.textContent = text;
  btn.addEventListener('click', e => { e.preventDefault(); e.stopPropagation(); setSubtreeLeaves(node, value); applyVisibility(); });
  return btn;
}
function renderNode(node, open) {
  if (node.is_leaf) {
    const row = document.createElement('div');
    row.className = 'tree-leaf';
    const lbl = document.createElement('span');
    lbl.className = 'tree-label leaf';
    lbl.textContent = node.label;
    lbl.title = node.name;
    row.append(checkbox(node.id), lbl);
    return row;
  }
  const det = document.createElement('details');
  det.open = open;
  const summary = document.createElement('summary');
  const lbl = document.createElement('span');
  lbl.className = 'tree-label';
  lbl.textContent = node.label + ' (' + countLeaves(node) + ')';
  summary.append(checkbox(node.id), lbl, subtreeButton(node, 'all', true), subtreeButton(node, 'none', false));
  const wrap = document.createElement('div');
  wrap.className = 'tree-branch-children';
  for (const c of node.children) wrap.appendChild(renderNode(c, false));
  det.append(summary, wrap);
  return det;
}
function setSubtreeLeaves(node, value) {
  if (node.is_leaf) setState(node.id, value); else node.children.forEach(c => setSubtreeLeaves(c, value));
}
function countLeaves(node) { return node.is_leaf ? 1 : node.children.reduce((s, c) => s + countLeaves(c), 0); }
const leafIds = new Set(Object.values(nameToId));
function setAllBranches(value) {
  for (const id in treeState) if (!leafIds.has(Number(id))) setState(Number(id), value);
  applyVisibility();
}
function setAllLeaves(value) { leafIds.forEach(id => setState(id, value)); applyVisibility(); }

const axes = { xaxis: 'log', yaxis: 'log' };
const layout = () => ({
  paper_bgcolor: '#111', plot_bgcolor: '#1a1a1a',
  font: { color: '#ddd', size: 12 },
  xaxis: { title: 'n', type: axes.xaxis, gridcolor: '#2e2e2e', color: '#aaa' },
  yaxis: { title: 'median time (us)', type: axes.yaxis, gridcolor: '#2e2e2e', color: '#aaa' },
  legend: { bgcolor: '#1a1a1a', bordercolor: '#444', borderwidth: 1, font: { size: 11 },
            itemclick: 'toggle', itemdoubleclick: 'toggleothers' },
  margin: { l: 70, r: 20, t: 10, b: 55 },
  hovermode: 'closest',
});
function toggleLog(axis) {
  axes[axis] = axes[axis] === 'log' ? 'linear' : 'log';
  Plotly.relayout('chart', { [axis + '.type']: axes[axis] });
}

const slider = document.getElementById('size-slider');
slider.max = NS.length - 1;
slider.value = -1;
function selectedSize() { const idx = parseInt(slider.value); return idx < 0 ? null : NS[idx]; }
function onSliderInput() {
  const n = selectedSize();
  document.getElementById('size-readout').textContent = n === null ? 'off' : 'n=' + n;
}
function timeAt(trace, n) { const k = trace.x.indexOf(n); return k < 0 ? null : trace.y[k]; }

let refName = null;
function setRef(name) {
  refName = name;
  document.getElementById('ref-input').value = name || '';
  document.getElementById('ref-readout').textContent = name || 'none';
  applyVisibility();
}
function onRefInputChange() {
  const v = document.getElementById('ref-input').value;
  setRef(nameToTrace[v] ? v : null);
}
// The reference itself always shows; an entry with no point at n is hidden.
function perfFilterPass(trace) {
  const mode = document.getElementById('perf-mode').value;
  const n = selectedSize();
  if (mode === 'off' || !refName || n === null || trace.name === refName) return true;
  const ref = timeAt(nameToTrace[refName], n), mine = timeAt(trace, n);
  if (ref === null) return true;
  if (mine === null) return false;
  return mode === 'faster' ? mine < ref : mine > ref;
}
function regex() {
  const v = document.getElementById('filter').value.trim();
  try { return v ? new RegExp(v, 'i') : null; } catch { return null; }
}
// SVG traces hover and draw better; past a few hundred of them only WebGL keeps up.
function visibleTraces() {
  const re = regex(), n = selectedSize();
  const shown = ALL.filter(t => leafVisible(t.name) && (!re || re.test(t.name)) && perfFilterPass(t));
  const type = shown.length > 200 ? 'scattergl' : 'scatter';
  return shown.map(t => ({ ...t, type, legendrank: n === null ? 0 : timeAt(t, n) ?? Infinity }));
}
function applyVisibility() {
  const shown = visibleTraces();
  document.getElementById('count').textContent = shown.length + ' / ' + ALL.length + ' visible';
  Plotly.react('chart', shown, layout(), { responsive: true });
}

Plotly.newPlot('chart', [], layout(), { responsive: true });
document.getElementById('chart').on('plotly_click', evt => { if (evt.points.length) setRef(evt.points[0].data.name); });
document.getElementById('tree').appendChild(renderNode(TREE, true));
applyVisibility();
onSliderInput();
</script>
</body>
</html>
"""


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="chart how every entry scales with n from timing archives, one per size")
    parser.add_argument("archives", type=Path, nargs="+", help="archives written by `--timing N K --out`")
    parser.add_argument("--out", type=Path, default=Path("target/bench_report.html"))
    args = parser.parse_args(argv)
    series = load(path.read_text() for path in args.archives)
    sizes = sorted({n for points in series.values() for n, _ in points})
    args.out.write_text(page(traces(series), tree(series), sizes))
    print(f"{len(series)} entries at n in {sizes}, report at {args.out}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
