# traits

Thin glue between the `sort_logger` crate's instrumentation traits and
the rest of the project. All algorithm registration lives elsewhere — see
[bench_registry.rs](../bench_registry.rs) and
[../sorts/README.md](../sorts/README.md).

## Files

- `log_traits.rs` — re-exports `SortLog`, `SortLogger`, `NoOpLogger`,
  `VisualizerLogger` from the `sort_logger` crate so the rest of the
  source tree can import them via `crate::traits::log_traits::…`.
- `sort_traits.rs` — `SortAlgo<T, U>` (the legacy four-method trait:
  `name`, `big_o`, `stable`, `sort`). Still used by a few hand-rolled
  callers but no longer the registration mechanism.
- `mod.rs` — re-exports the above. Also defines a `SortFn` type alias
  (`fn(&mut [usize], &mut NoOpLogger)`) that the per-family registration
  helpers in `sorts/{shell,circle,comb,rod}_sorts/` use for ergonomics.
  Not a registry key.

## What is _not_ here

The `SORT_REGISTRY` / `SORT_VIS_REGISTRY` hash maps, the `create_sort!`
macro, and the `#[derive(SortRegistry)]` proc macro that previous
revisions had here are gone. Dispatch goes through
`bench_registry::ALGORITHMS` (one `linkme` distributed-slice covering
every category). The navigation tree comes from
`sort_registry_core::SortTree`, populated by each algorithm's per-leaf
ctor calling `register_sort_path`.
