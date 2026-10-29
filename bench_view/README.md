# `bench_view`

The benchmark page that `plot_bench.py` writes, in parts the script inlines
into one self contained HTML file with the archives' data.

- `page.html` is the shell: the layout, the light and dark colour tokens and
  the slots the data and the two scripts go into.
- `view.js` is the page's logic with no page in it: the facets the entries'
  kinds, families and axes form, the counts beside each value, the colour
  slots, best of a facet, dominance over a size range, the effect of a
  facet, and the view as a URL hash.
- `page.js` reads the controls, keeps the state and draws the three views
  with Plotly: scaling, time against size with a panel per input shape;
  effects, what each value of a facet costs against the best value in its
  context; and parallel coordinates over every facet that varies.
- `view.test.js` tests `view.js` under node, `node --test
  bench_view/view.test.js`; the `scripts` step of `verify.sh` runs it.

Dominance is judged over a size range, since an entry that is fastest on
small inputs is often too slow to reach large ones. Within the range, one
entry dominates another when, on every input shape the other reached there,
it reaches at least as far, is never slower by more than the tolerance at a
size both have, and is faster on the geometric mean of those sizes. An entry
that never entered the range is dominated by any of its kind that did.
