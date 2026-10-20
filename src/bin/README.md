# bin

Auxiliary binaries beside the default visualiser.

- `compare_sorts.rs`: the performance gate. Hand rolled u64 quick, merge and heap sorts against the generic equivalents under `NoOpLogger`; each pair must stay within 1.05x. Arguments: size, runs, margin.
- `measure_random_shell.rs`: complexity probe and correctness check for the random shell sort variants.
