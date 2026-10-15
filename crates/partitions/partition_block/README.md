# `partition_block`

Block partition: classifies elements into offset buffers block by block, then swaps in a tight loop. Declares scratch of twice the block size.

Catalog fragment `partition_block.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `pks_block` | `Block` | `block` | PartScheme |  |
