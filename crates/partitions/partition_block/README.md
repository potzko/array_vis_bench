# `partition_block`

Block partition: classifies elements into offset buffers block by block, then swaps in a tight loop. Declares scratch of twice the block size.

Its catalog components are declared in `partition_block.spec`.
