# `small_sort_insertion`

Insertion sort for sub arrays up to a threshold, over an insertion strategy. Declared at 16 and 32, linear and binary, under the small sort, non trivial, fun and kind roles.

Catalog fragment `small_sort_insertion.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `nt_insertion32` | `InsertionSmallSort<LinearInsertion, 32>` | `insertion: 32` | NonTrivialSmallSort |  |
| `fs_ss_insertion32` | `InsertionSmallSort<LinearInsertion, 32>` | `insertion: 32` | FunSmallSort |  |
| `ss_insertion32` | `InsertionSmallSort<LinearInsertion, 32>` | `insertion: 32` | SmallSort |  |
| `sk_ins_lin16` | `InsertionSmallSort<LinearInsertion, 16>` | `insertion: 16` | SmallSortKind | SmallSort |
| `sk_ins_lin32` | `InsertionSmallSort<LinearInsertion, 32>` | `insertion: 32` | SmallSortKind | SmallSort |
| `sk_ins_bin16` | `InsertionSmallSort<BinaryInsertion, 16>` | `binary insertion: 16` | SmallSortKind | SmallSort |
| `sk_ins_bin32` | `InsertionSmallSort<BinaryInsertion, 32>` | `binary insertion: 32` | SmallSortKind | SmallSort |
