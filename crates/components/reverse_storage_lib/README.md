# `reverse_storage_lib`

How weak heap sort stores its per node reverse bit: one byte per node or bit packed. Writes go through the logger.

Catalog fragment `reverse_storage_lib.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `weak_byte_storage` | `ByteStorage` | `byte storage` | ReverseStorage |  |
| `weak_bit_storage` | `BitStorage` | `bit storage` | ReverseStorage |  |
