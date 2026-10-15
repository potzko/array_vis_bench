# `src`

Source of `sort_logger`. The crate README one level up says what it is for.

- `lib.rs`: Event log for sorting algorithms.
- `loggers.rs`: Reference `SortLogger` implementations: `NoOpLogger` and `VisualizerLogger`.
- `sort_log.rs`: `SortLog` event variants and the `arr_name!` helper that identifies an array by its base pointer for cross-event correlation.
- `sort_logger.rs`: The `SortLogger` trait, every sort algorithm in the workspace calls into one of its methods to record an observable operation.
