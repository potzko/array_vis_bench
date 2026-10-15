# `array_vis_bench_cli`

The interactive visualiser: the faceted picker, the input, size and render
prompts, and the render call that writes `output.mp4`. It reads the registry
by name and function pointer only, so any link root that populates
`ALGORITHMS` can call `run`.

- `src/lib.rs`: `run`, the picker over `sort_registry_core::all_variants`, and the breadcrumb and label rendering.
