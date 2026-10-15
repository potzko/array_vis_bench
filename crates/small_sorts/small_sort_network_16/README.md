# `small_sort_network_16`

Batcher's odd even merge network for sixteen elements, chaining to the eight element network for shorter inputs.

Catalog fragment `small_sort_network_16.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `nt_network16` | `Network16SmallSort` | `network: 16` | NonTrivialSmallSort |  |
| `fs_ss_network16` | `Network16SmallSort` | `network: 16` | FunSmallSort |  |
| `ss_network16` | `Network16SmallSort` | `network: 16` | SmallSort |  |
| `sk_network16` | `Network16SmallSort` | `network: 16` | SmallSortKind | SmallSort |
