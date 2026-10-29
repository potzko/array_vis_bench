// Tests of the facet page's logic under node: the facets the entries form,
// the live counts, the colour slots, best of a facet, dominance over a size
// range, the effect of a facet and the view as a URL hash.
// Run with `node --test bench_view/`.
const test = require('node:test');
const assert = require('node:assert/strict');
const View = require('./view.js');

// Entries as the archives describe them, with their facet values filled in.
function described(list) {
  return list.map(([name, kind, family, axes]) => {
    const entry = { name, kind, family, axes };
    entry.values = View.facetValues(entry);
    return entry;
  });
}

const QUICK = described([
  ['quick sort<part: hoare, small: ins>', 'sort', 'quick sort', [['Part', 'hoare', 'part'], ['Small', 'ins', 'small']]],
  ['quick sort<part: hoare, small: net>', 'sort', 'quick sort', [['Part', 'hoare', 'part'], ['Small', 'net', 'small']]],
  ['quick sort<part: lomuto, small: ins>', 'sort', 'quick sort', [['Part', 'lomuto', 'part'], ['Small', 'ins', 'small']]],
  ['quick sort<part: lomuto, small: net>', 'sort', 'quick sort', [['Part', 'lomuto', 'part'], ['Small', 'net', 'small']]],
  ['merge sort<small: ins>', 'sort', 'merge sort', [['Small', 'ins', 'small']]],
  ['rotation<gries>', 'rotation', 'rotation', []],
]);

const names = entries => entries.map(e => e.name);

test('an entry carries its kind, family and every axis as facet values', () => {
  assert.deepEqual(QUICK[0].values, { kind: 'sort', family: 'quick sort', 'Part|part': 'hoare', 'Small|small': 'ins' });
});

test('the facets are kind and family first, then the axes the most entries carry, values in natural order', () => {
  const facets = View.facets(described([
    ['a', 'sort', 'a', [['Small', 'insertion: 32', 'small'], ['Pivot', 'first', 'part/pivot/a'], ['Pivot', 'mid', 'part/pivot/b']]],
    ['b', 'sort', 'b', [['Small', 'insertion: 4', 'small'], ['Pivot', 'mid', 'part/pivot/a']]],
    ['c', 'sort', 'c', [['Small', 'insertion: 4', 'small']]],
  ]));
  assert.deepEqual(facets.map(f => f.key), ['kind', 'family', 'Small|small', 'Pivot|part/pivot/a', 'Pivot|part/pivot/b']);
  // A role met at one path is labelled by the role alone, at several by role and path.
  assert.deepEqual(facets.map(f => f.label), ['kind', 'family', 'Small', 'Pivot (part/pivot/a)', 'Pivot (part/pivot/b)']);
  assert.deepEqual(facets[2].values, ['insertion: 4', 'insertion: 32']);
});

test('an entry shows when none of its values is switched off, and a facet it lacks does not hide it', () => {
  const off = { 'Part|part': new Set(['lomuto']) };
  assert.deepEqual(names(QUICK.filter(e => View.passes(e.values, off))), [
    'quick sort<part: hoare, small: ins>', 'quick sort<part: hoare, small: net>', 'merge sort<small: ins>', 'rotation<gries>',
  ]);
});

test('a value counts the entries it would show given every other facet', () => {
  const off = { 'Part|part': new Set(['lomuto']), 'Small|small': new Set(['net']) };
  const counts = View.counts(QUICK.map(e => e.values), off);
  // Switching lomuto on would add the one lomuto entry that is not also a net one.
  assert.deepEqual(counts['Part|part'], { hoare: 1, lomuto: 1 });
  assert.deepEqual(counts['Small|small'], { ins: 2, net: 1 });
  assert.deepEqual(counts.family, { 'quick sort': 1, 'merge sort': 1, rotation: 1 });
});

test('a colour slot follows its group through a filter and the groups past the slots fold to other', () => {
  const first = View.assignSlots(['a', 'b', 'c'], new Map(), 2);
  assert.deepEqual([...first], [['a', 0], ['b', 1], ['c', -1]]);
  // `a` filtered out: `b` keeps its slot, `c` takes the freed one.
  const second = View.assignSlots(['b', 'c'], first, 2);
  assert.deepEqual([...second], [['b', 1], ['c', 0]]);
  // Two groups remembered at one slot: the first keeps it, the other moves.
  const clash = View.assignSlots(['a', 'c'], new Map([['a', 0], ['c', 0]]), 2);
  assert.deepEqual([...clash], [['a', 0], ['c', 1]]);
});

test('best of a facet keeps the fastest entry of each group equal on every other facet', () => {
  const time = { 'quick sort<part: hoare, small: ins>': 5, 'quick sort<part: hoare, small: net>': 4,
    'quick sort<part: lomuto, small: ins>': 7, 'quick sort<part: lomuto, small: net>': 9, 'merge sort<small: ins>': 3 };
  const score = e => time[e.name] ?? Infinity;
  assert.deepEqual(names(View.bestOf(QUICK, 'Small|small', score)), [
    'quick sort<part: hoare, small: net>', 'quick sort<part: lomuto, small: ins>', 'merge sort<small: ins>', 'rotation<gries>',
  ]);
  // Best of family groups entries equal on everything else across families.
  assert.deepEqual(names(View.bestOf(QUICK, 'family', score)).length, QUICK.length);
});

test('best of a facet folds the paths nested under it into the choice', () => {
  const entries = described([
    ['q<dual first>', 'sort', 'q', [['Part', 'dual', 'part'], ['Pivot', 'first', 'part/pivot']]],
    ['q<dual mid>', 'sort', 'q', [['Part', 'dual', 'part'], ['Pivot', 'mid', 'part/pivot']]],
    ['q<block>', 'sort', 'q', [['Part', 'block', 'part']]],
  ]);
  const score = e => ({ 'q<dual first>': 3, 'q<dual mid>': 2, 'q<block>': 1 })[e.name];
  assert.deepEqual(names(View.bestOf(entries, 'Part|part', score)), ['q<block>']);
});

// Points per input as the page holds them: x sizes, y times.
const ranged = (series, lo = 1, hi = 1e9) => View.rangeOf(series, lo, hi);

test('an entry dominates another that it beats at every shared size within the range', () => {
  const fast = ranged({ shuffled: { x: [1, 10, 100], y: [1, 10, 100] } });
  const slow = ranged({ shuffled: { x: [1, 10, 100], y: [2, 20, 200] } });
  assert.equal(View.dominates(fast, slow, 0.1), true);
  assert.equal(View.dominates(slow, fast, 0.1), false);
  // Within the tolerance both ways, the faster on the geometric mean wins, and
  // an exact tie dominates neither way.
  const close = ranged({ shuffled: { x: [1, 10], y: [1.05, 9.5] } });
  const base = ranged({ shuffled: { x: [1, 10], y: [1, 10] } });
  assert.equal(View.dominates(close, base, 0.1), true);
  assert.equal(View.dominates(base, close, 0.1), false);
  assert.equal(View.dominates(base, base, 0.1), false);
});

test('a quadratic sort that stops early is dominated only over a range past where it wins', () => {
  // Insertion is faster to 30 and stops at 300; quick goes on to 10000.
  const insertion = { shuffled: { x: [10, 30, 100, 300], y: [1, 5, 50, 500] } };
  const quick = { shuffled: { x: [10, 30, 100, 300, 1000, 10000], y: [3, 8, 30, 100, 400, 5000] } };
  assert.equal(View.dominates(ranged(quick), ranged(insertion), 0.1), false);
  assert.equal(View.dominates(ranged(insertion), ranged(quick), 0.1), false);
  // From 100 up quick is faster wherever both ran and reaches further.
  assert.equal(View.dominates(ranged(quick, 100), ranged(insertion, 100), 0.1), true);
  // Up to 30 insertion beats quick, and both reach 30.
  assert.equal(View.dominates(ranged(insertion, 1, 30), ranged(quick, 1, 30), 0.1), true);
  // Past 1000 insertion never entered the range, so anything that did dominates it.
  assert.equal(View.dominates(ranged(quick, 1000), ranged(insertion, 1000), 0.1), true);
  assert.equal(View.dominates(ranged(insertion, 1000), ranged(quick, 1000), 0.1), false);
});

test('an entry that stops short of another cannot dominate it, however fast', () => {
  const short = ranged({ shuffled: { x: [1, 10], y: [1, 1] } });
  const long = ranged({ shuffled: { x: [1, 10, 100], y: [5, 50, 500] } });
  assert.equal(View.dominates(short, long, 0.1), false);
});

test('dominance holds on every input the dominated entry was timed on', () => {
  const a = ranged({ shuffled: { x: [10], y: [1] }, sorted: { x: [10], y: [9] } });
  const b = ranged({ shuffled: { x: [10], y: [2] }, sorted: { x: [10], y: [3] } });
  assert.equal(View.dominates(a, b, 0.1), false);
  assert.equal(View.dominates(ranged({ shuffled: { x: [10], y: [1] } }), b, 0.1), false);
});

test('only an entry of the same scope can dominate', () => {
  const entries = described([
    ['a', 'sort', 'f', []], ['b', 'sort', 'f', []], ['c', 'sort', 'g', []],
  ]);
  const rangedOf = { a: ranged({ s: { x: [1], y: [1] } }), b: ranged({ s: { x: [1], y: [5] } }), c: ranged({ s: { x: [1], y: [9] } }) };
  const kept = View.undominated(entries, e => rangedOf[e.name], 0.1, e => e.family);
  assert.deepEqual(names(kept), ['a', 'c']);
  assert.deepEqual(names(View.undominated(entries, e => rangedOf[e.name], 0.1, e => e.kind)), ['a']);
});

test('the effect of a facet is each value over the best in its context', () => {
  const time = { 'quick sort<part: hoare, small: ins>': 4, 'quick sort<part: hoare, small: net>': 2,
    'quick sort<part: lomuto, small: ins>': 9, 'quick sort<part: lomuto, small: net>': 3, 'merge sort<small: ins>': 7 };
  const effects = View.effects(QUICK, 'Small|small', e => time[e.name]);
  // Two quick sort contexts, hoare and lomuto: net is best in both, ins is 2x
  // and 3x. Merge sort has one entry, so no context compares two values.
  assert.deepEqual(effects.map(({ family, value, contexts, min, max }) => [family, value, contexts, min, max]), [
    ['quick sort', 'ins', 2, 2, 3],
    ['quick sort', 'net', 2, 1, 1],
  ]);
  assert.ok(Math.abs(effects[0].mean - Math.sqrt(6)) < 1e-12);
});

test('the view goes to a hash and back, and a broken hash gives the defaults', () => {
  const defaults = { off: {}, colour: 'family', n: 1000, log: true };
  const state = { off: { 'Part|part': new Set(['lomuto', 'hoare']) }, colour: 'Small|small', n: 1000, log: false };
  const hash = View.encodeState(state, defaults);
  assert.deepEqual(View.decodeState(hash, defaults), state);
  assert.deepEqual(View.decodeState('#nonsense', defaults), defaults);
  // A field of the wrong type keeps its default.
  assert.deepEqual(View.decodeState(View.encodeState({ ...defaults, n: 'x' }, { ...defaults, n: 0 }), defaults), defaults);
  // Only what differs from the defaults goes into the hash.
  assert.equal(View.encodeState(defaults, defaults), '');
});
