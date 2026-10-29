// The facet page's logic, free of the page. `facets` lists what the entries'
// kinds, families and axes let a reader filter on, `passes` and `counts` say
// which entries the switched off values hide and how many each value would
// bring back, `assignSlots` keeps colours on their groups, `bestOf` collapses
// a facet to its fastest value, `dominates` and `undominated` judge entries
// over a size range, `effects` measures what a facet's values cost, and
// `encodeState` and `decodeState` carry the view in the URL hash. `page.js`
// draws what these return, `view.test.js` tests them under node.
const View = (() => {
  const collator = new Intl.Collator(undefined, { numeric: true });
  const byName = (a, b) => collator.compare(a, b);

  const axisKey = (role, path) => role + '|' + path;
  const pathOf = key => key.includes('|') ? key.slice(key.indexOf('|') + 1) : null;

  // An entry's value at every facet it carries, keyed by facet: its kind,
  // its family and each axis by role and path.
  function facetValues(entry) {
    const values = { kind: entry.kind, family: entry.family };
    for (const [role, value, path] of entry.axes) values[axisKey(role, path)] = value;
    return values;
  }

  // Every facet the entries carry, kind and family first, then the axes in
  // name order, each with its values in natural order. A role met at one
  // path is labelled by the role alone, so a role two families share reads
  // as one facet.
  function facets(entries) {
    const valuesOf = new Map();
    for (const entry of entries) {
      for (const [key, value] of Object.entries(entry.values)) {
        if (!valuesOf.has(key)) valuesOf.set(key, new Set());
        valuesOf.get(key).add(value);
      }
    }
    const axes = [...valuesOf.keys()].filter(key => pathOf(key) !== null).sort(byName);
    const paths = new Map();
    for (const key of axes) {
      const role = key.slice(0, key.indexOf('|'));
      paths.set(role, (paths.get(role) || 0) + 1);
    }
    const label = key => {
      const role = key.slice(0, key.indexOf('|'));
      return paths.get(role) > 1 ? role + ' (' + pathOf(key) + ')' : role;
    };
    return ['kind', 'family', ...axes]
      .filter(key => valuesOf.has(key))
      .map(key => ({ key, label: pathOf(key) === null ? key : label(key), values: [...valuesOf.get(key)].sort(byName) }));
  }

  // The facets whose value in `values` is switched off in `off`, a map from
  // facet key to the set of values switched off there.
  function failing(values, off) {
    return Object.keys(values).filter(key => off[key] && off[key].has(values[key]));
  }

  function passes(values, off) { return failing(values, off).length === 0; }

  // For each facet and value, how many entries would show with that value
  // switched on: an entry hidden by one facet alone counts toward its own
  // value there, an entry hidden by two counts nowhere.
  function counts(valuesList, off) {
    const out = {};
    const add = (key, value) => {
      out[key] = out[key] || {};
      out[key][value] = (out[key][value] || 0) + 1;
    };
    for (const values of valuesList) {
      const fails = failing(values, off);
      if (fails.length === 0) for (const key of Object.keys(values)) add(key, values[key]);
      else if (fails.length === 1) add(fails[0], values[fails[0]]);
    }
    return out;
  }

  // A colour slot per group, keeping the slot a group had in `previous` so a
  // filter never repaints a group that stays. New groups take the free slots
  // in order; groups past the `slots` get -1, the muted other.
  function assignSlots(groups, previous, slots) {
    const out = new Map();
    const taken = new Set();
    for (const group of groups) {
      const slot = previous.get(group);
      if (slot !== undefined && slot >= 0 && slot < slots) { out.set(group, slot); taken.add(slot); }
    }
    let next = 0;
    for (const group of groups) {
      if (out.has(group)) continue;
      while (taken.has(next)) next++;
      if (next < slots) { out.set(group, next); taken.add(next); } else out.set(group, -1);
    }
    return new Map(groups.map(group => [group, out.get(group)]));
  }

  // The key of the context an entry sits in with respect to facet `key`:
  // every value it carries except at `key` and at the paths nested under it.
  function contextOf(values, key) {
    const path = pathOf(key);
    const nested = other => path !== null && pathOf(other) !== null && pathOf(other).startsWith(path + '/');
    const kept = Object.keys(values).filter(other => other !== key && !nested(other)).sort();
    return JSON.stringify(kept.map(other => [other, values[other]]));
  }

  // The entries equal on every facet but `key` and the paths under it
  // collapse to the one with the lowest `score`, the first on a tie, so a
  // facet's choice and every choice nested in it fold into one. Entries
  // without `key` pass through. The order of `entries` is kept.
  function bestOf(entries, key, score) {
    const best = new Map();
    for (const entry of entries) {
      if (!(key in entry.values)) continue;
      const context = contextOf(entry.values, key);
      const s = score(entry);
      const held = best.get(context);
      if (held === undefined || s < held.score) best.set(context, { entry, score: s });
    }
    const winners = new Set([...best.values()].map(held => held.entry));
    return entries.filter(entry => !(key in entry.values) || winners.has(entry));
  }

  // An entry's points between sizes `lo` and `hi`, per input: the largest
  // size reached and the time at each size. `series` maps an input to its
  // `{x, y}`; an input with no point in the range is left out.
  function rangeOf(series, lo, hi) {
    const out = new Map();
    for (const [input, { x, y }] of Object.entries(series)) {
      const at = new Map();
      x.forEach((n, i) => { if (n >= lo && n <= hi && y[i] !== null) at.set(n, y[i]); });
      if (at.size) out.set(input, { max: Math.max(...at.keys()), at });
    }
    return out;
  }

  // Whether `a` dominates `b`, each as `rangeOf` gives it. On every input `b`
  // reached in the range, `a` must reach at least as far, be no slower than
  // `b` by more than `tol` at any size both have, and over all those sizes
  // be faster on the geometric mean, so a tie dominates neither way. An
  // entry that never entered the range is dominated by any that did: it was
  // too slow to get there in its share of the time, or capped below it.
  function dominates(a, b, tol) {
    if (b.size === 0) return a.size > 0;
    let compared = 0, log = 0;
    for (const [input, pb] of b) {
      const pa = a.get(input);
      if (!pa || pa.max < pb.max) return false;
      for (const [n, tb] of pb.at) {
        const ta = pa.at.get(n);
        if (ta === undefined) continue;
        if (ta > tb * (1 + tol)) return false;
        log += Math.log(ta / tb);
        compared++;
      }
    }
    return compared > 0 && log < 0;
  }

  // The entries no other entry of the same scope dominates; `ranged(entry)`
  // gives its points as `rangeOf` does and `scope(entry)` the group it is
  // judged in, so a sort is never measured against a rotation.
  function undominated(entries, ranged, tol, scope) {
    const points = new Map(entries.map(entry => [entry, ranged(entry)]));
    return entries.filter(b => !entries.some(a => a !== b && scope(a) === scope(b) && dominates(points.get(a), points.get(b), tol)));
  }

  const geomean = list => Math.exp(list.reduce((sum, r) => sum + Math.log(r), 0) / list.length);

  // What each value of facet `key` costs, per family that carries it: in
  // every context, the entries of the family equal on all other facets, each
  // value's time over the context's best, summarised as the geometric mean,
  // least and most over the contexts where two or more entries were timed.
  // `time(entry)` is undefined for an entry without a time.
  function effects(entries, key, time) {
    const contexts = new Map();
    for (const entry of entries) {
      const t = time(entry);
      if (!(key in entry.values) || t === undefined || t === null || !isFinite(t)) continue;
      const context = contextOf(entry.values, key);
      if (!contexts.has(context)) contexts.set(context, []);
      contexts.get(context).push({ entry, t });
    }
    const ratios = new Map();
    for (const members of contexts.values()) {
      if (members.length < 2) continue;
      const best = Math.min(...members.map(m => m.t));
      for (const { entry, t } of members) {
        const id = JSON.stringify([entry.values.family, entry.values[key]]);
        if (!ratios.has(id)) ratios.set(id, []);
        ratios.get(id).push(t / best);
      }
    }
    return [...ratios.entries()]
      .map(([id, list]) => {
        const [family, value] = JSON.parse(id);
        return { family, value, mean: geomean(list), min: Math.min(...list), max: Math.max(...list), contexts: list.length };
      })
      .sort((p, q) => byName(p.family, q.family) || byName(p.value, q.value));
  }

  // The view as a URL hash: the fields that differ from `defaults`, with the
  // switched off values as lists, as JSON. Empty when nothing differs.
  function encodeState(state, defaults) {
    const out = {};
    for (const [field, value] of Object.entries(state)) {
      if (field === 'off') {
        const off = Object.fromEntries(Object.entries(value).filter(([, set]) => set.size).map(([key, set]) => [key, [...set]]));
        if (Object.keys(off).length) out.off = off;
      } else if (value !== defaults[field]) out[field] = value;
    }
    return Object.keys(out).length ? encodeURIComponent(JSON.stringify(out)) : '';
  }

  // The view a hash describes, over `defaults`: a field of another type than
  // its default, or a hash that does not parse, keeps the default.
  function decodeState(hash, defaults) {
    const state = { ...defaults, off: {} };
    let parsed;
    try { parsed = JSON.parse(decodeURIComponent(hash.replace(/^#/, ''))); } catch { return state; }
    if (parsed === null || typeof parsed !== 'object') return state;
    for (const [field, value] of Object.entries(parsed)) {
      if (field === 'off' && value && typeof value === 'object') {
        for (const [key, list] of Object.entries(value)) {
          if (Array.isArray(list)) state.off[key] = new Set(list.filter(v => typeof v === 'string'));
        }
      } else if (field in defaults && field !== 'off' && typeof value === typeof defaults[field]) state[field] = value;
    }
    return state;
  }

  return { facetValues, facets, passes, counts, assignSlots, bestOf, rangeOf, dominates, undominated, effects, encodeState, decodeState };
})();

if (typeof module !== 'undefined') module.exports = View;
