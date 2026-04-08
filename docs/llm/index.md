# LLM Guide

You are working on `array_vis_bench` — a sorting algorithm benchmark and visualisation framework in Rust. This folder contains instructions specifically for LLM assistants.

Read these files in order:

1. **[context-loading.md](context-loading.md)** — which files to read first, what to always have loaded, and how to efficiently orient yourself in this codebase.
2. **[patterns.md](patterns.md)** — coding patterns, conventions, and anti-patterns specific to this project. Follow these when writing new code.
3. **[documentation-maintenance.md](documentation-maintenance.md)** — rules for keeping docs and READMEs up to date when you make changes.

## Quick orientation

- This is a Cargo workspace with 5 crates. The root crate (`array_vis_bench`) contains all sort implementations.
- Every sort goes through a `SortLogger` — this is the central abstraction. Read `sort_logger/src/sort_logger.rs` before touching any sort code.
- Sorts self-register at startup via macros + `#[ctor]` hooks. There is no central sort list.
- The `docs/` folder (not `docs/llm/`) has human-facing architecture docs. Both must be kept in sync.
