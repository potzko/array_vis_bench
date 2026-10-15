# `small_sort_basic`

The sentinel and tiny small sorts: no small sort (threshold zero), size one and size two.

Catalog fragment `small_sort_basic.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `nt_size2` | `Size2SmallSort` | `size: 2` | NonTrivialSmallSort |  |
| `ss_size1` | `Size1SmallSort` | `size: 1` | SmallSort |  |
| `ss_size2` | `Size2SmallSort` | `size: 2` | SmallSort |  |
| `sk_size1` | `Size1SmallSort` | `size: 1` | SmallSortKind | SmallSort |
| `sk_size2` | `Size2SmallSort` | `size: 2` | SmallSortKind | SmallSort |
