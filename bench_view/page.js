// The benchmark page: the facet sidebar, the controls, and three views of
// the entries the filters let through, drawn with Plotly. Scaling plots time
// against size with one panel per input shape, Effects plots what each value
// of a facet costs against the best value in its context, and Parallel lays
// every varying facet and the time side by side. `DATA` is the archives as
// `plot_bench.py` joined them and `View` holds every rule the page applies,
// so this file reads the controls, keeps the state and draws.
(() => {
  const $ = id => document.getElementById(id);
  const collator = new Intl.Collator(undefined, { numeric: true });
  const SLOTS = 8;

  const entries = DATA.entries.map((entry, index) => ({ ...entry, index, values: View.facetValues(entry), series: {} }));
  for (const [index, input, x, y] of DATA.series) entries[index].series[input] = { x, y };
  const SIZES = DATA.sizes, INPUTS = DATA.inputs;
  const FACETS = View.facets(entries);
  const facetOf = new Map(FACETS.map(f => [f.key, f]));
  const entryNamed = new Map(entries.map(e => [e.name, e]));

  const middleSize = SIZES.includes(1000) ? 1000 : SIZES[Math.floor(SIZES.length / 2)];
  const DEFAULTS = {
    off: {}, re: '', tab: 'scaling', n: middleSize, colour: 'kind', best: '', effect: '', input: INPUTS[0],
    dom: 'off', lo: SIZES[0], hi: SIZES[SIZES.length - 1], tol: 10, ref: '', perf: 'off', norm: false, logx: true, logy: true,
  };

  // A state read from a hash may name sizes, inputs or facets this data
  // does not have; those fall back to the defaults.
  function readState() {
    const state = View.decodeState(location.hash, DEFAULTS);
    const keys = new Set(FACETS.map(f => f.key));
    if (!SIZES.includes(state.n)) state.n = DEFAULTS.n;
    if (!SIZES.includes(state.lo)) state.lo = DEFAULTS.lo;
    if (!SIZES.includes(state.hi)) state.hi = DEFAULTS.hi;
    if (!INPUTS.includes(state.input)) state.input = DEFAULTS.input;
    if (state.colour !== 'entry' && !keys.has(state.colour)) state.colour = DEFAULTS.colour;
    if (state.best && !keys.has(state.best)) state.best = '';
    if (state.effect && !keys.has(state.effect)) state.effect = '';
    if (!['off', 'family', 'kind'].includes(state.dom)) state.dom = 'off';
    if (!['scaling', 'effects', 'parallel'].includes(state.tab)) state.tab = 'scaling';
    if (!['off', 'faster', 'slower'].includes(state.perf)) state.perf = 'off';
    return state;
  }
  let state = readState();

  const css = name => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  // Plotly reads a few tags in its text, so a name such as `block<first>`
  // goes in with its brackets as entities.
  const esc = text => String(text).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

  const offSet = key => state.off[key] || new Set();
  const inputsOn = () => INPUTS.filter(input => !offSet('input').has(input));
  function regex() {
    try { return state.re ? new RegExp(state.re, 'i') : null; } catch { return null; }
  }
  function timeAt(entry, input, n) {
    const series = entry.series[input];
    if (!series) return undefined;
    const k = series.x.indexOf(n);
    return k < 0 ? undefined : series.y[k];
  }

  // The entries the name filter and the inputs let through, before the facets.
  function candidates() {
    const re = regex(), on = inputsOn();
    return entries.filter(e => (!re || re.test(e.name)) && on.some(input => e.series[input]));
  }

  // Best of a facet ranks an entry by the geometric mean of its time at n
  // over the inputs on; an entry that did not reach n there ranks last.
  function score(entry) {
    const times = inputsOn().filter(input => entry.series[input]).map(input => timeAt(entry, input, state.n));
    if (!times.length || times.some(t => t === undefined)) return Infinity;
    return Math.exp(times.reduce((sum, t) => sum + Math.log(t), 0) / times.length);
  }

  // Dominance compares every pair, so its outcome is kept until the entries
  // or its settings change.
  let dominance = { key: '', kept: new Set() };
  function undominated(list) {
    if (state.dom === 'off') return list;
    const on = inputsOn();
    const key = [state.dom, state.lo, state.hi, state.tol, on.join('\t'), list.map(e => e.index).join(',')].join('|');
    if (dominance.key !== key) {
      const ranged = e => View.rangeOf(Object.fromEntries(on.filter(input => e.series[input]).map(input => [input, e.series[input]])), state.lo, state.hi);
      const scope = state.dom === 'family' ? e => e.kind + '\t' + e.family : e => e.kind;
      dominance = { key, kept: new Set(View.undominated(list, ranged, state.tol / 100, scope)) };
    }
    return list.filter(e => dominance.kept.has(e));
  }

  // Faster or slower than the reference at n on every input on where both
  // have a time; an entry with no such input is hidden, the reference never.
  function againstReference(list) {
    const ref = entryNamed.get(state.ref);
    if (state.perf === 'off' || !ref) return list;
    return list.filter(e => {
      if (e === ref) return true;
      const pairs = inputsOn().map(input => [timeAt(e, input, state.n), timeAt(ref, input, state.n)]).filter(([a, b]) => a !== undefined && b !== undefined);
      return pairs.length > 0 && pairs.every(([a, b]) => state.perf === 'faster' ? a < b : a > b);
    });
  }

  function shown() {
    const passing = candidates().filter(e => View.passes(e.values, state.off));
    const best = state.best ? View.bestOf(passing, state.best, score) : passing;
    return { passing, list: againstReference(undominated(best)) };
  }

  // The sidebar: a checklist per facet that varies among the entries the
  // other facets let through, or that has values switched off; the facets
  // fixed at one value are named beneath instead.
  const openFacets = new Set(['input', 'kind', 'family']);
  function element(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }
  function button(text, act, className) {
    const node = element('button', className, text);
    node.addEventListener('click', event => { event.preventDefault(); event.stopPropagation(); act(); });
    return node;
  }
  function setOff(key, values) { state.off[key] = new Set(values); render(); }
  function toggle(key, value) {
    const off = new Set(offSet(key));
    if (off.has(value)) off.delete(value); else off.add(value);
    setOff(key, off);
  }

  function renderSidebar(cands) {
    const counts = View.counts(cands.map(e => e.values), state.off);
    const re = regex();
    const facetPassing = entries.filter(e => (!re || re.test(e.name)) && View.passes(e.values, state.off));
    counts.input = Object.fromEntries(INPUTS.map(input => [input, facetPassing.filter(e => e.series[input]).length]));
    const all = [{ key: 'input', label: 'input', values: INPUTS }, ...FACETS];
    const live = f => f.values.filter(v => (counts[f.key] || {})[v] > 0);
    const varies = f => offSet(f.key).size > 0 || live(f).length >= 2;
    const box = $('facets');
    const scroll = box.scrollTop;
    box.replaceChildren(...all.filter(varies).map(f => facetElement(f, counts[f.key] || {})));
    box.scrollTop = scroll;
    const fixed = all.filter(f => !varies(f) && live(f).length === 1).map(f => f.label + ' ' + live(f)[0]);
    $('fixed').textContent = fixed.length ? 'Fixed here: ' + fixed.join(', ') : '';
  }

  function facetElement(f, counts) {
    const off = offSet(f.key);
    const details = element('details');
    details.open = openFacets.has(f.key);
    details.addEventListener('toggle', () => { if (details.open) openFacets.add(f.key); else openFacets.delete(f.key); });
    const summary = element('summary');
    summary.append(
      element('span', 'facet-label', f.label),
      element('span', 'facet-meta', f.values.filter(v => !off.has(v)).length + ' of ' + f.values.length),
      button('all', () => setOff(f.key, [])),
      button('none', () => setOff(f.key, f.values)),
    );
    const list = element('div', 'values');
    for (const value of f.values) {
      const count = counts[value] || 0;
      const row = element('div', 'value' + (count ? '' : ' empty'));
      const check = element('input');
      check.type = 'checkbox';
      check.checked = !off.has(value);
      check.addEventListener('click', event => { event.stopPropagation(); toggle(f.key, value); });
      const label = element('span', 'value-label', value);
      label.title = value;
      row.append(check, label, element('span', 'count', String(count)), button('only', () => setOff(f.key, f.values.filter(v => v !== value)), 'only'));
      row.addEventListener('click', () => toggle(f.key, value));
      list.append(row);
    }
    details.append(summary, list);
    const section = element('section', 'facet');
    section.append(details);
    return section;
  }

  // The controls, filled once, then set from the state on every render.
  function fill(select, options) {
    select.replaceChildren(...options.map(([value, text]) => { const o = element('option', '', text); o.value = value; return o; }));
  }
  const facetOptions = FACETS.map(f => [f.key, f.label]);
  fill($('colour'), [['entry', 'entry'], ...facetOptions]);
  fill($('best'), [['', 'none'], ...facetOptions.filter(([key]) => key !== 'kind')]);
  fill($('effect'), [['', 'first that varies'], ...facetOptions.filter(([key]) => key !== 'kind')]);
  fill($('one-input'), INPUTS.map(input => [input, input]));
  fill($('lo'), SIZES.map(n => [String(n), String(n)]));
  fill($('hi'), SIZES.map(n => [String(n), String(n)]));
  $('size').max = SIZES.length - 1;
  $('names').replaceChildren(...entries.map(e => { const o = element('option'); o.value = e.name; return o; }));

  function syncControls() {
    $('filter').value = state.re;
    $('size').value = SIZES.indexOf(state.n);
    $('size-readout').textContent = '= ' + state.n;
    $('colour').value = state.colour;
    $('best').value = state.best;
    $('effect').value = state.effect;
    $('one-input').value = state.input;
    $('dom').value = state.dom;
    $('lo').value = String(state.lo);
    $('hi').value = String(state.hi);
    $('tol').value = state.tol;
    $('ref').value = state.ref;
    $('perf').value = state.perf;
    $('norm').checked = state.norm;
    $('logx').setAttribute('aria-pressed', state.logx);
    $('logy').setAttribute('aria-pressed', state.logy);
    for (const tab of document.querySelectorAll('[data-tab]')) tab.setAttribute('aria-pressed', tab.dataset.tab === state.tab);
    for (const node of document.querySelectorAll('[data-tabs]')) node.classList.toggle('hidden', !node.dataset.tabs.split(' ').includes(state.tab));
  }

  const on = (id, event, act) => $(id).addEventListener(event, () => { act($(id)); render(); });
  let typing;
  $('filter').addEventListener('input', () => { clearTimeout(typing); typing = setTimeout(() => { state.re = $('filter').value; render(); }, 200); });
  on('size', 'input', node => { state.n = SIZES[Number(node.value)]; });
  on('colour', 'change', node => { state.colour = node.value; });
  on('best', 'change', node => { state.best = node.value; });
  on('effect', 'change', node => { state.effect = node.value; });
  on('one-input', 'change', node => { state.input = node.value; });
  on('dom', 'change', node => { state.dom = node.value; });
  on('lo', 'change', node => { state.lo = Number(node.value); });
  on('hi', 'change', node => { state.hi = Number(node.value); });
  on('tol', 'change', node => { state.tol = Math.max(0, Number(node.value) || 0); });
  on('ref', 'change', node => { state.ref = entryNamed.has(node.value) ? node.value : ''; });
  on('ref-clear', 'click', () => { state.ref = ''; });
  on('perf', 'change', node => { state.perf = node.value; });
  on('norm', 'change', node => { state.norm = node.checked; });
  on('logx', 'click', () => { state.logx = !state.logx; });
  on('logy', 'click', () => { state.logy = !state.logy; });
  for (const tab of document.querySelectorAll('[data-tab]')) tab.addEventListener('click', () => { state.tab = tab.dataset.tab; render(); });
  $('copy').addEventListener('click', () => {
    navigator.clipboard.writeText(location.href).then(() => { $('copy').textContent = 'Copied'; setTimeout(() => { $('copy').textContent = 'Copy link'; }, 1200); });
  });

  // Plot chrome in the page's colours, read from the theme tokens each draw.
  function layout() {
    return {
      paper_bgcolor: css('--surface'), plot_bgcolor: css('--surface'),
      font: { family: 'system-ui, -apple-system, Segoe UI, sans-serif', size: 12, color: css('--ink-2') },
      margin: { l: 64, r: 16, t: 34, b: 48 }, hovermode: 'closest',
      hoverlabel: { bgcolor: css('--raised'), bordercolor: css('--border'), font: { color: css('--ink') } },
      legend: { bgcolor: css('--surface'), font: { size: 11 } },
    };
  }
  function axis(title, type) {
    return { title: { text: title }, type, gridcolor: css('--grid'), linecolor: css('--axis'), zerolinecolor: css('--axis'), color: css('--muted') };
  }
  function note(text) {
    return { text, xref: 'paper', yref: 'paper', x: 0.5, y: 0.5, showarrow: false, font: { size: 14, color: css('--muted') } };
  }

  // Scaling: a panel per input on, sharing both axes. The lines are one
  // trace per colour group with gaps between entries, so a thousand entries
  // draw as a handful of traces; groups past the palette fold into one
  // muted other, and the reference is drawn on top in the ink colour.
  let slots = new Map(), slotsOf = '';
  const groupOf = e => state.colour === 'entry' ? e.name : (e.values[state.colour] ?? '(none)');
  function drawScaling(list) {
    const panels = inputsOn().filter(input => list.some(e => e.series[input]));
    const groups = [...new Set(list.map(groupOf))].sort(collator.compare);
    if (slotsOf !== state.colour) { slots = new Map(); slotsOf = state.colour; }
    slots = new Map([...slots, ...View.assignSlots(groups, slots, SLOTS)]);
    const colours = Array.from({ length: SLOTS }, (_, i) => css('--s' + (i + 1)));
    const ref = entryNamed.get(state.ref);
    const norm = state.norm && ref;
    const pointCount = list.reduce((sum, e) => sum + panels.reduce((s, input) => s + (e.series[input] ? e.series[input].x.length : 0), 0), 0);
    const type = pointCount > 20000 ? 'scattergl' : 'scatter';
    const traces = [], annotations = [], shapes = [];
    // A group enters the legend on the first panel it has lines in.
    const named = new Set();
    const legend = key => !named.has(key) && named.add(key);
    const view = layout();
    const columns = Math.min(panels.length, 3);
    const rows = Math.ceil(panels.length / 3) || 1;
    view.grid = { rows, columns: columns || 1, pattern: 'independent', xgap: 0.07, ygap: 0.16 };
    panels.forEach((input, p) => {
      const suffix = p ? String(p + 1) : '';
      const refAt = norm && ref.series[input] ? new Map(ref.series[input].x.map((n, i) => [n, ref.series[input].y[i]])) : null;
      const lines = new Map();
      const add = (key, e, series) => {
        if (!lines.has(key)) lines.set(key, { x: [], y: [], text: [], customdata: [], count: 0 });
        const line = lines.get(key);
        series.x.forEach((n, i) => {
          line.x.push(n);
          line.y.push(norm ? (refAt && refAt.has(n) ? series.y[i] / refAt.get(n) : null) : series.y[i]);
          line.text.push(esc(e.name));
          line.customdata.push(e.index);
        });
        line.x.push(null); line.y.push(null); line.text.push(null); line.customdata.push(null);
        line.count++;
      };
      for (const e of list) if (e.series[input]) add(slots.get(groupOf(e)) >= 0 ? groupOf(e) : '\u0000other', e, e.series[input]);
      const hover = '%{text}<br>n = %{x}<br>' + (norm ? '%{y:.3f}x the reference' : '%{y:.4g} us') + '<extra></extra>';
      const trace = (key, line, extra) => ({
        type, mode: line.x.length / Math.max(line.count, 1) > 20 ? 'lines' : 'lines+markers',
        x: line.x, y: line.y, text: line.text, customdata: line.customdata, connectgaps: false,
        xaxis: 'x' + suffix, yaxis: 'y' + suffix, legendgroup: key, showlegend: legend(key), hovertemplate: hover, ...extra,
      });
      const other = lines.get('\u0000other');
      if (other) traces.push(trace('other', other, { name: 'other (' + other.count + ')', opacity: 0.45, line: { color: css('--muted'), width: 1 }, marker: { color: css('--muted'), size: 3 } }));
      for (const group of groups) {
        const line = lines.get(group);
        if (!line) continue;
        const colour = colours[slots.get(group)];
        traces.push(trace(group, line, { name: esc(group), line: { color: colour, width: 2 }, marker: { color: colour, size: 5 } }));
      }
      if (ref && ref.series[input]) {
        const line = { x: [], y: [], text: [], customdata: [], count: 0 };
        lines.set('\u0000ref', line);
        add('\u0000ref', ref, ref.series[input]);
        traces.push(trace('reference', lines.get('\u0000ref'), { name: 'reference', line: { color: css('--ink'), width: 2, dash: 'dash' }, marker: { color: css('--ink'), size: 4 } }));
      }
      const bottom = Math.floor(p / 3) === rows - 1 || p + 3 >= panels.length;
      view['xaxis' + suffix] = { ...axis(bottom ? 'n' : '', state.logx ? 'log' : 'linear'), ...(p ? { matches: 'x' } : {}) };
      view['yaxis' + suffix] = { ...axis(p % 3 === 0 ? (norm ? 'time over the reference' : 'time (us)') : '', state.logy ? 'log' : 'linear'), ...(p ? { matches: 'y' } : {}) };
      annotations.push({ text: esc(input), xref: 'x' + suffix + ' domain', yref: 'y' + suffix + ' domain', x: 0, y: 1.02, xanchor: 'left', yanchor: 'bottom', showarrow: false, font: { color: css('--ink'), size: 12 } });
      shapes.push({ type: 'line', xref: 'x' + suffix, yref: 'y' + suffix + ' domain', x0: state.n, x1: state.n, y0: 0, y1: 1, line: { color: css('--muted'), width: 1, dash: 'dot' } });
    });
    if (!panels.length) annotations.push(note('Every entry is filtered out'));
    Plotly.react('chart', traces, { ...view, annotations, shapes }, { responsive: true, displaylogo: false });
  }

  // Effects: per family, each value of the facet against the best value in
  // its context, the geometric mean as the dot and the least and most over
  // the contexts as the whisker, each family offset within the value's row.
  // Values run best first down the chart. The whiskers are line traces in
  // the dots' offset group, so each sits on its own family's dot.
  let familySlots = new Map();
  function defaultEffect(passing) {
    const varying = FACETS.filter(f => f.key !== 'kind' && f.key !== 'family' && new Set(passing.map(e => e.values[f.key]).filter(v => v !== undefined)).size >= 2);
    return varying.length ? varying[0].key : 'family';
  }
  function drawEffects(passing) {
    const key = state.effect || defaultEffect(passing);
    const facet = facetOf.get(key);
    const rows = View.effects(passing, key, e => timeAt(e, state.input, state.n));
    const families = [...new Set(rows.map(r => r.family))].sort(collator.compare);
    familySlots = new Map([...familySlots, ...View.assignSlots(families, familySlots, SLOTS)]);
    const colours = Array.from({ length: SLOTS }, (_, i) => css('--s' + (i + 1)));
    const valueMeans = new Map();
    for (const r of rows) valueMeans.set(r.value, [...(valueMeans.get(r.value) || []), r.mean]);
    const geomean = list => Math.exp(list.reduce((sum, m) => sum + Math.log(m), 0) / list.length);
    const order = [...valueMeans.keys()].sort((a, b) => geomean(valueMeans.get(a)) - geomean(valueMeans.get(b)));
    const traces = families.flatMap(family => {
      const mine = rows.filter(r => r.family === family);
      const slot = familySlots.get(family);
      const colour = slot >= 0 ? colours[slot] : css('--muted');
      const shared = { type: 'scatter', orientation: 'h', offsetgroup: family, legendgroup: family };
      return [{
        ...shared, mode: 'lines', showlegend: false, hoverinfo: 'skip',
        x: mine.flatMap(r => [r.min, r.max, null]), y: mine.flatMap(r => [esc(r.value), esc(r.value), null]),
        line: { color: colour, width: 1.5 },
      }, {
        ...shared, mode: 'markers', name: esc(family),
        x: mine.map(r => r.mean), y: mine.map(r => esc(r.value)),
        marker: { color: colour, size: 9, line: { color: css('--surface'), width: 2 } },
        customdata: mine.map(r => [r.contexts, r.min, r.max]),
        hovertemplate: esc(family) + '<br>%{y}: %{x:.3f}x the best in its context<br>%{customdata[1]:.3f}x to %{customdata[2]:.3f}x over %{customdata[0]} contexts<extra></extra>',
      }];
    });
    const view = layout();
    view.scattermode = 'group';
    view.scattergap = 0.4;
    view.margin = { ...view.margin, l: 200, t: 40 };
    view.xaxis = axis('time over the best value in its context', 'log');
    view.yaxis = { ...axis('', 'category'), categoryorder: 'array', categoryarray: order.map(esc), autorange: 'reversed' };
    const title = 'Effect of ' + facet.label + ' on ' + state.input + ' at n = ' + state.n;
    view.annotations = [{ text: esc(title), xref: 'paper', yref: 'paper', x: 0, y: 1.01, xanchor: 'left', yanchor: 'bottom', showarrow: false, font: { color: css('--ink'), size: 13 } }];
    if (!rows.length) view.annotations.push(note('No context has two entries timed at n = ' + state.n + ' on ' + state.input));
    Plotly.react('chart', traces, view, { responsive: true, displaylogo: false });
  }

  // Parallel: one axis per facet that varies among the shown entries, then
  // the time at n on the chosen input, each line coloured by that time.
  function drawParallel(list) {
    const timed = list.filter(e => timeAt(e, state.input, state.n) !== undefined);
    const value = (e, key) => e.values[key] ?? '-';
    const dims = FACETS.filter(f => new Set(timed.map(e => value(e, f.key))).size >= 2).slice(0, 12).map(f => {
      const values = [...new Set(timed.map(e => value(e, f.key)))].sort(collator.compare);
      const index = new Map(values.map((v, i) => [v, i]));
      return {
        label: esc(f.label), values: timed.map(e => index.get(value(e, f.key))), range: [0, values.length - 1],
        tickvals: values.map((_, i) => i), ticktext: values.map(v => esc(v.length > 22 ? v.slice(0, 21) + '.' : v)),
      };
    });
    const logs = timed.map(e => Math.log10(timeAt(e, state.input, state.n)));
    dims.push({ label: 'log10 time (us)', values: logs });
    const view = layout();
    view.margin = { ...view.margin, l: 120, r: 80, t: 64 };
    const traces = timed.length ? [{
      type: 'parcoords', dimensions: dims,
      line: { color: logs, colorscale: [[0, css('--seq-low')], [1, css('--seq-high')]], showscale: true, colorbar: { title: { text: 'log10 us' } } },
      labelfont: { color: css('--ink'), size: 12 }, tickfont: { color: css('--ink-2'), size: 10 }, rangefont: { color: css('--muted'), size: 9 },
    }] : [];
    view.annotations = timed.length ? [] : [note('No shown entry has a time at n = ' + state.n + ' on ' + state.input)];
    Plotly.react('chart', traces, view, { responsive: true, displaylogo: false });
  }

  function render() {
    const cands = candidates();
    renderSidebar(cands);
    const { passing, list } = shown();
    syncControls();
    $('count').textContent = list.length + ' of ' + entries.length + ' entries';
    if (state.tab === 'scaling') drawScaling(list);
    else if (state.tab === 'effects') drawEffects(passing);
    else drawParallel(list);
    const hash = View.encodeState(state, DEFAULTS);
    history.replaceState(null, '', hash ? '#' + hash : location.pathname + location.search);
  }

  render();
  const chart = $('chart');
  // The chart's box changes without the window resizing, when a tab hides
  // or shows a row of controls, and may not have its size yet at the first
  // draw, so Plotly is told whenever the box changes.
  new ResizeObserver(() => Plotly.Plots.resize(chart)).observe(chart);
  // A legend item stands for a facet value, so clicking it switches that
  // value off; with colour by entry it makes the entry the reference.
  chart.on('plotly_legendclick', event => {
    const group = event.data[event.curveNumber].legendgroup;
    if (state.tab === 'effects') toggle('family', group);
    else if (state.tab === 'scaling' && group !== 'other' && group !== 'reference' && group !== '(none)') {
      if (state.colour === 'entry') { state.ref = group; render(); } else toggle(state.colour, group);
    }
    return false;
  });
  chart.on('plotly_click', event => {
    const point = event.points[0];
    if (state.tab !== 'scaling' || !point || point.customdata === null || point.customdata === undefined) return;
    state.ref = entries[point.customdata].name;
    render();
  });
  window.addEventListener('hashchange', () => { state = readState(); render(); });
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', render);
})();
