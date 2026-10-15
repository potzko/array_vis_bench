# `small_sort_deferred_insertion`

Deferred insertion: the quicksort stops at short sub arrays and one final insertion pass, linear or binary, cleans up. Declared at thresholds 16 and 32.

Catalog fragment `small_sort_deferred_insertion.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `dss_lin16` | `DeferredInsertion<LinearInsertion, 16>` | `deferred insertion: 16` | DeferredSmallSort |  |
| `dss_lin32` | `DeferredInsertion<LinearInsertion, 32>` | `deferred insertion: 32` | DeferredSmallSort |  |
| `dss_bin16` | `DeferredInsertion<BinaryInsertion, 16>` | `deferred binary insertion: 16` | DeferredSmallSort |  |
| `dss_bin32` | `DeferredInsertion<BinaryInsertion, 32>` | `deferred binary insertion: 32` | DeferredSmallSort |  |
