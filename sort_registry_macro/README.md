# sort_registry_macro

Proc-macro crate exposing the legacy `sort_family!` macro. New cross-product families use `combo_codegen::family!` (in the `combo_codegen` crate) instead; this macro stays in the workspace because several simple families still use it.

## What it contains

- **`sort_family!`** — a declarative macro that emits one `bench_registry::AlgorithmEntry` (`Category::Sort`) per leaf of a small variant tree. Each leaf gets:
  - a `#[linkme::distributed_slice(bench_registry::ALGORITHMS)]` static so the entry shows up in the global algorithm registry,
  - a `#[ctor::ctor]` hook that calls `sort_registry_core::register_sort_path` so the variant appears in the interactive navigation tree.

## Syntax

```rust
sort_registry_macro::sort_family! {
    type Sort = BubbleSort;
    name        = "bubble sort";
    big_o       = "O(N^2)";
    stable      = true;
    direct_sort = true;
    path        = ["bubble sorts", "bubble sort"];
}
```

For parameterised families, declare slots and their concrete variants inline:

```rust
sort_registry_macro::sort_family! {
    type Sort = MySort<{Strategy}>;

    Strategy {
        StrategyA => "a"
        StrategyB => "b"
    }

    name        = "my sort {Strategy}";
    big_o       = "O(N log N)";
    stable      = false;
    direct_sort = true;
    path        = ["my sorts", "{Strategy}"];
}
```

## When to use which macro

| Macro | Crate | Best for |
|---|---|---|
| `combo_codegen::family!` | `combo_codegen` | Cross-products with multiple slots that draw from project-wide `component!`-annotated types (preferred for new families). |
| `sort_registry_macro::sort_family!` | this crate | Single-leaf or self-contained variant trees that don't need cross-crate component scanning. |

Both register into the same `bench_registry::ALGORITHMS` slice and produce identical menu paths.
