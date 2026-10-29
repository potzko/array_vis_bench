# `bench_view`

The benchmark page that `plot_bench.py` writes, in three parts the script
inlines into one self contained HTML file.

- `view.js` is the page's logic with no page in it: the facets the entries'
  kinds, families and axes form, the counts beside each value, the colour
  slots, best of a facet, dominance over a size range, the effect of a
  facet, and the view as a URL hash.
- `view.test.js` tests that logic under node, `node --test
  bench_view/view.test.js`; the `scripts` step of `verify.sh` runs it.
