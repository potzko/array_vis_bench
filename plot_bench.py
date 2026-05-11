#!/usr/bin/env python3
"""
Parse the bench archive emitted by `cargo bench --bench sorts` and
generate an interactive N-scaling chart with a tree-based selector.

Usage:
    python3 plot_bench.py [target/bench_archive.json]
    # Then open bench_report.html in a browser.
"""

import json
import sys
from pathlib import Path
from collections import defaultdict


def parse(archive_path: Path):
    """
    Returns (data, tree) where:
      data = {sort_name: [(n, mean_ns, std_err_ns), ...]}  (sorted by n)
      tree = nested {label, is_leaf, name?, children}
    """
    if not archive_path.exists():
        print(f"No benchmark archive at {archive_path}", file=sys.stderr)
        return {}, None

    with open(archive_path) as f:
        archive = json.load(f)

    data = defaultdict(list)
    for r in archive.get("results", []):
        data[r["name"]].append((r["n"], r["mean_ns"], r["stderr_ns"]))
    for name in data:
        data[name].sort(key=lambda t: t[0])

    return dict(data), archive.get("tree"), archive.get("thresholds", [])


def generate(data: dict, tree: dict, thresholds: list, output: Path):
    traces = []
    all_ns: set = set()
    for name, points in sorted(data.items()):
        ns = [p[0] for p in points]
        all_ns.update(ns)
        traces.append({
            "name": name,
            "x": ns,
            "y": [p[1] / 1_000 for p in points],   # ns → µs
            "error_y": {"type": "data",
                        "array": [p[2] / 1_000 for p in points],
                        "visible": True},
            "mode": "lines+markers",
            "type": "scattergl",   # WebGL — much faster for many traces
        })

    sorted_ns = sorted(all_ns)
    # Cutoff step-line: y holds at threshold(N) until next N level. line.shape='hv'
    # draws horizontal-then-vertical, which is exactly the adaptive threshold's behavior.
    cutoff_trace = {
        "name": "drop cutoff",
        "x": [t["n"] for t in thresholds],
        "y": [t["threshold_ns"] / 1_000 for t in thresholds],  # ns → µs
        "mode": "lines",
        "type": "scattergl",
        "line": {"color": "#ff4444", "dash": "dash", "shape": "hv", "width": 2},
        "hovertemplate": "drop cutoff at N=%{x}: %{y:.0f} µs<extra></extra>",
    }

    traces_json  = json.dumps(traces)
    tree_json    = json.dumps(tree or {"label": "all", "is_leaf": False, "children": []})
    ns_json      = json.dumps(sorted_ns)
    cutoff_json  = json.dumps(cutoff_trace)

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Sort benchmark — N scaling</title>
<script src="https://cdn.plot.ly/plotly-2.27.0.min.js"></script>
<style>
  * {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{ font-family: sans-serif; background: #111; color: #eee;
          display: flex; height: 100vh; padding: 10px; gap: 10px; }}

  #sidebar {{ width: 360px; display: flex; flex-direction: column; gap: 8px;
              min-height: 0; }}
  #sidebar h2 {{ font-size: 1em; font-weight: 500; color: #bbb;
                 border-bottom: 1px solid #333; padding-bottom: 4px; }}
  #sidebar-buttons {{ display: flex; gap: 6px; flex-wrap: wrap; }}
  #tree {{ flex: 1; min-height: 0; overflow: auto;
           background: #1a1a1a; border: 1px solid #2e2e2e; border-radius: 4px;
           padding: 6px 8px; font-size: 12px; }}

  details {{ margin-left: 0; }}
  details > summary {{ list-style: none; cursor: pointer;
                       display: flex; align-items: center; gap: 4px;
                       padding: 1px 0; user-select: none; }}
  details > summary::-webkit-details-marker {{ display: none; }}
  details > summary::before {{
    content: "▶"; font-size: 9px; color: #666; width: 10px; flex-shrink: 0;
    transition: transform 0.1s;
  }}
  details[open] > summary::before {{ transform: rotate(90deg); }}
  .tree-leaf {{ display: flex; align-items: center; gap: 4px;
                padding: 1px 0 1px 14px; }}
  .tree-branch-children {{ margin-left: 12px;
                           border-left: 1px solid #2a2a2a;
                           padding-left: 4px; }}
  .tree-label {{ flex: 1; cursor: pointer; color: #ccc;
                 white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }}
  .tree-label.leaf {{ color: #888; font-size: 11px; }}
  input[type=checkbox] {{ flex-shrink: 0; accent-color: #4a9eff; cursor: pointer; }}
  .subtree-btn {{ font-size: 10px; padding: 0 5px; height: 16px; line-height: 14px;
                  background: #2a2a2a; border: 1px solid #444; border-radius: 2px;
                  color: #aaa; cursor: pointer; flex-shrink: 0; }}
  .subtree-btn:hover {{ background: #3a3a3a; color: #fff; }}

  #main {{ flex: 1; display: flex; flex-direction: column; gap: 8px;
           min-width: 0; min-height: 0; }}
  h1 {{ font-size: 1.05em; font-weight: 500; }}
  #controls {{ display: flex; gap: 10px; align-items: center; flex-wrap: wrap;
               background: #1a1a1a; border: 1px solid #2e2e2e; border-radius: 4px;
               padding: 8px 10px; font-size: 12px; }}
  #controls input[type=text] {{ background: #222; color: #eee; border: 1px solid #555;
             padding: 4px 8px; border-radius: 3px; font-size: 12px; width: 280px; }}
  #controls input[type=range] {{ width: 200px; }}
  button {{ background: #2a2a2a; color: #ddd; border: 1px solid #555;
            padding: 4px 10px; border-radius: 3px; cursor: pointer; font-size: 12px; }}
  button:hover {{ background: #3a3a3a; }}
  .ctl-label {{ color: #888; }}
  #count {{ color: #888; margin-left: auto; }}
  #speed-readout {{ color: #4a9eff; font-variant-numeric: tabular-nums; min-width: 70px; }}
  .ctl-readout {{ color: #4a9eff; font-variant-numeric: tabular-nums;
                  max-width: 380px; overflow: hidden; text-overflow: ellipsis;
                  white-space: nowrap; }}
  select {{ background: #222; color: #eee; border: 1px solid #555;
            padding: 4px 8px; border-radius: 3px; font-size: 12px; }}
  #ref-input {{ width: 320px; }}
  #chart {{ flex: 1; min-height: 0; }}
</style>
</head>
<body>

<div id="sidebar">
  <h2>Sort tree</h2>
  <div id="sidebar-buttons">
    <button onclick="setAllBranches(true)">Branches on</button>
    <button onclick="setAllBranches(false)">Branches off</button>
    <button onclick="setAllLeaves(true)">Leaves on</button>
    <button onclick="setAllLeaves(false)">Leaves off</button>
  </div>
  <div id="tree"></div>
</div>

<div id="main">
  <h1>Sort benchmark — mean time (µs) vs N</h1>
  <div id="controls">
    <input id="filter" type="text" placeholder="Regex filter…" oninput="applyVisibility()">
    <button onclick="document.getElementById('filter').value=''; applyVisibility()">Clear filter</button>

    <span class="ctl-label">N =</span>
    <input id="speed-slider" type="range" min="-1" step="1" oninput="onSliderInput()" onchange="onSliderChange()">
    <span id="speed-readout">off</span>

    <button onclick="toggleLogY()">log Y</button>
    <button onclick="toggleLogX()">log X</button>
    <span id="count"></span>
  </div>
  <div id="controls">
    <span class="ctl-label">Show sorts</span>
    <select id="perf-mode" onchange="onPerfModeChange()">
      <option value="off" selected>— off —</option>
      <option value="faster">faster than</option>
      <option value="slower">slower than</option>
    </select>
    <input id="ref-input" list="ref-options" type="text"
           placeholder="ref sort (type to search, or click a trace)"
           oninput="onRefInputChange()">
    <button onclick="clearRef()">Clear ref</button>
    <span class="ctl-label">at the selected N. Ref:</span>
    <span id="ref-readout" class="ctl-readout">—</span>
  </div>
  <div id="chart"></div>
</div>

<datalist id="ref-options"></datalist>

<script>
const ALL    = {traces_json};
const TREE   = {tree_json};
const NS     = {ns_json};
const CUTOFF = {cutoff_json};

// Fast lookup: trace name → trace object
const nameToTrace = Object.create(null);
ALL.forEach(t => nameToTrace[t.name] = t);

// Populate the autocomplete datalist once.
(() => {{
  const dl = document.getElementById('ref-options');
  for (const t of ALL) {{
    const o = document.createElement('option');
    o.value = t.name;
    dl.appendChild(o);
  }}
}})();

// ── Tree state ─────────────────────────────────────────────────────────
// Branches default to true, leaves default to false. A leaf is "visible"
// when itself AND every ancestor branch are true. So clicking a leaf
// enables it directly (its ancestors are already on); use the per-branch
// "all/none" buttons to bulk-toggle a subtree's leaves.
const treeState     = Object.create(null);  // {{ id: bool }}
const parentOf      = Object.create(null);  // {{ id: parentId }}
const nameToId      = Object.create(null);  // {{ leafName: id }}
const nodeById      = Object.create(null);  // {{ id: TreeNode }}
const checkboxById  = Object.create(null);  // {{ id: HTMLInputElement }}
let _nextId = 0;

function walkTree(node, parentId) {{
  const id = _nextId++;
  parentOf[id]  = parentId;
  nodeById[id]  = node;
  treeState[id] = !node.is_leaf;         // branch=true, leaf=false
  if (node.is_leaf && node.name) nameToId[node.name] = id;
  for (const c of (node.children || [])) walkTree(c, id);
  node._id = id;
  return id;
}}
walkTree(TREE, -1);

function leafVisible(name) {{
  let id = nameToId[name];
  if (id === undefined) return true;     // trace with no tree node → always show
  while (id !== -1) {{
    if (!treeState[id]) return false;
    id = parentOf[id];
  }}
  return true;
}}

// ── Tree rendering ─────────────────────────────────────────────────────
function renderTree() {{
  const root = document.getElementById('tree');
  root.innerHTML = '';
  root.appendChild(renderNode(TREE, true));
}}

function renderNode(node, openByDefault) {{
  const id = node._id;
  if (node.is_leaf) {{
    const row = document.createElement('div');
    row.className = 'tree-leaf';
    const cb = document.createElement('input');
    cb.type = 'checkbox';
    cb.checked = treeState[id];
    cb.addEventListener('change', () => {{ treeState[id] = cb.checked; applyVisibility(); }});
    checkboxById[id] = cb;
    const lbl = document.createElement('span');
    lbl.className = 'tree-label leaf';
    lbl.textContent = node.label;
    lbl.title = node.name || node.label;
    row.appendChild(cb);
    row.appendChild(lbl);
    return row;
  }} else {{
    const det = document.createElement('details');
    if (openByDefault) det.open = true;
    const summary = document.createElement('summary');
    const cb = document.createElement('input');
    cb.type = 'checkbox';
    cb.checked = treeState[id];
    cb.addEventListener('click', e => e.stopPropagation());
    cb.addEventListener('change', () => {{ treeState[id] = cb.checked; applyVisibility(); }});
    checkboxById[id] = cb;
    const lbl = document.createElement('span');
    lbl.className = 'tree-label';
    const leafCount = countLeaves(node);
    lbl.textContent = node.label + ' (' + leafCount + ')';

    const onBtn = document.createElement('button');
    onBtn.className = 'subtree-btn';
    onBtn.textContent = 'all';
    onBtn.title = 'Enable every leaf under this branch';
    onBtn.addEventListener('click', e => {{
      e.preventDefault(); e.stopPropagation();
      setSubtreeLeaves(node, true);
      applyVisibility();
    }});
    const offBtn = document.createElement('button');
    offBtn.className = 'subtree-btn';
    offBtn.textContent = 'none';
    offBtn.title = 'Disable every leaf under this branch';
    offBtn.addEventListener('click', e => {{
      e.preventDefault(); e.stopPropagation();
      setSubtreeLeaves(node, false);
      applyVisibility();
    }});

    summary.appendChild(cb);
    summary.appendChild(lbl);
    summary.appendChild(onBtn);
    summary.appendChild(offBtn);
    det.appendChild(summary);
    const wrap = document.createElement('div');
    wrap.className = 'tree-branch-children';
    for (const c of (node.children || [])) wrap.appendChild(renderNode(c, false));
    det.appendChild(wrap);
    return det;
  }}
}}

// Recursively flip every leaf under `node` to `value`, updating both
// `treeState` and the live checkbox elements (no full re-render needed).
function setSubtreeLeaves(node, value) {{
  if (node.is_leaf) {{
    treeState[node._id] = value;
    const cb = checkboxById[node._id];
    if (cb) cb.checked = value;
  }} else {{
    for (const c of (node.children || [])) setSubtreeLeaves(c, value);
  }}
}}

function countLeaves(node) {{
  if (node.is_leaf) return 1;
  let s = 0; for (const c of (node.children || [])) s += countLeaves(c);
  return s;
}}

// ── Bulk tree actions ──────────────────────────────────────────────────
const leafIdSet = new Set(Object.values(nameToId));

function setAllBranches(value) {{
  for (const id in treeState) {{
    const numId = Number(id);
    if (!leafIdSet.has(numId)) {{
      treeState[numId] = value;
      const cb = checkboxById[numId];
      if (cb) cb.checked = value;
    }}
  }}
  applyVisibility();
}}
function setAllLeaves(value) {{
  for (const name in nameToId) {{
    const id = nameToId[name];
    treeState[id] = value;
    const cb = checkboxById[id];
    if (cb) cb.checked = value;
  }}
  applyVisibility();
}}

// ── Chart state ────────────────────────────────────────────────────────
let logY = false, logX = true;

const LAYOUT = () => ({{
  paper_bgcolor: '#111', plot_bgcolor: '#1a1a1a',
  font: {{ color: '#ddd', size: 12 }},
  xaxis: {{ title: 'N (array size)', type: logX ? 'log' : 'linear', gridcolor: '#2e2e2e', color: '#aaa' }},
  yaxis: {{ title: 'mean time (µs)',  type: logY ? 'log' : 'linear', gridcolor: '#2e2e2e', color: '#aaa' }},
  legend: {{ bgcolor: '#1a1a1a', bordercolor: '#444', borderwidth: 1,
             font: {{ size: 11 }}, itemclick: 'toggle', itemdoubleclick: 'toggleothers' }},
  margin: {{ l: 70, r: 20, t: 10, b: 55 }},
  hovermode: 'closest',
}});

// Only the traces that should be visible are ever handed to Plotly. The
// cutoff step-line is always appended last (always shown, never toggled).
const CUTOFF_PINNED = {{...CUTOFF, visible: true, showlegend: true, legendrank: -1e9}};

Plotly.newPlot('chart', [CUTOFF_PINNED], LAYOUT(), {{responsive: true}});

// ── Filter regex ───────────────────────────────────────────────────────
function regex() {{
  const v = document.getElementById('filter').value.trim();
  if (!v) return null;
  try {{ return new RegExp(v, 'i'); }} catch {{ return null; }}
}}

// ── Performance filter ─────────────────────────────────────────────────
// "Show only sorts {{faster|slower}} than <ref> at the slider's N."
// Uses the same N as the speed-sort slider. Slider at -1 (off) ⇒ filter
// is disabled regardless of mode.
const perfFilter = {{ refName: null, mode: 'off' }};

function perfFilterPass(traceName) {{
  if (perfFilter.mode === 'off' || !perfFilter.refName) return true;
  if (traceName === perfFilter.refName) return true;   // ref itself always shown
  const idx = parseInt(slider.value);
  if (idx < 0) return true;
  const n = NS[idx];

  const refTrace = nameToTrace[perfFilter.refName];
  if (!refTrace) return true;
  const refK = refTrace.x.indexOf(n);
  if (refK < 0) return true;          // ref has no data at N — can't compare

  const myTrace = nameToTrace[traceName];
  if (!myTrace) return false;
  const myK = myTrace.x.indexOf(n);
  if (myK < 0) return false;          // hide sorts with no data at N

  const refY = refTrace.y[refK];
  const myY  = myTrace.y[myK];
  return perfFilter.mode === 'faster' ? myY < refY : myY > refY;
}}

function setRef(name) {{
  perfFilter.refName = name;
  document.getElementById('ref-input').value = name || '';
  const out = document.getElementById('ref-readout');
  out.textContent = name || '—';
  out.title = name || '';
  applyVisibility();
}}
function clearRef() {{ setRef(null); }}
function onRefInputChange() {{
  const v = document.getElementById('ref-input').value;
  setRef(v && nameToTrace[v] ? v : null);
}}
function onPerfModeChange() {{
  perfFilter.mode = document.getElementById('perf-mode').value;
  applyVisibility();
}}

// ── Visibility apply: tree + regex + perf filter ───────────────────────
// Builds the list of traces Plotly should see and hands it to react.
// Plotly only ever processes what's actually on screen.
function getVisibleTraces() {{
  const re = regex();
  const idx = parseInt(slider.value);
  const sortByN = (idx >= 0) ? NS[idx] : null;

  const out = [];
  for (const t of ALL) {{
    if (!leafVisible(t.name)) continue;
    if (re && !re.test(t.name)) continue;
    if (!perfFilterPass(t.name)) continue;
    if (sortByN !== null) {{
      const k = t.x.indexOf(sortByN);
      out.push({{...t, legendrank: k < 0 ? Infinity : t.y[k]}});
    }} else {{
      out.push(t);
    }}
  }}
  return out;
}}

function applyVisibility() {{
  const traces = getVisibleTraces();
  document.getElementById('count').textContent =
    traces.length + ' / ' + ALL.length + ' visible';
  traces.push(CUTOFF_PINNED);
  Plotly.react('chart', traces, LAYOUT(), {{responsive: true}});
}}

// ── Log toggles ────────────────────────────────────────────────────────
function toggleLogY() {{ logY = !logY; Plotly.relayout('chart', {{ 'yaxis.type': logY ? 'log' : 'linear' }}); }}
function toggleLogX() {{ logX = !logX; Plotly.relayout('chart', {{ 'xaxis.type': logX ? 'log' : 'linear' }}); }}

// ── Sort legend by speed at chosen N ───────────────────────────────────
// Slider value -1 = off (natural order). 0..NS.length-1 picks an N.
const slider = document.getElementById('speed-slider');
slider.min = -1;
slider.max = NS.length - 1;
slider.value = -1;

function onSliderInput() {{
  const idx = parseInt(slider.value);
  document.getElementById('speed-readout').textContent =
    idx < 0 ? 'off' : ('N=' + NS[idx]);
}}
function onSliderChange() {{
  // Both legendrank and perf-filter pivot on the slider's N — one rebuild does both.
  applyVisibility();
}}

// Click on a data point sets it as the perf-filter reference.
document.getElementById('chart').on('plotly_click', (evt) => {{
  if (!evt || !evt.points || !evt.points.length) return;
  const p = evt.points[0];
  if (!p.data || p.data.name === 'drop cutoff') return;
  setRef(p.data.name);
}});

// ── Initial render ─────────────────────────────────────────────────────
renderTree();
applyVisibility();
onSliderInput();
</script>
</body>
</html>"""

    output.write_text(html)
    print(f"Report written → {output}")


def main():
    archive = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("target/bench_archive.json")
    data, tree, thresholds = parse(archive)

    if not data:
        print("No data found. Run `cargo bench --bench sorts` first.")
        sys.exit(1)

    total = sum(len(v) for v in data.values())
    print(f"Loaded {len(data)} sorts, {total} data points, {len(thresholds)} threshold points.")
    generate(data, tree, thresholds, Path("bench_report.html"))


if __name__ == "__main__":
    main()
