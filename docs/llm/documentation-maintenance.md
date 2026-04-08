# Documentation Maintenance

When you make changes to the codebase, you must keep the documentation in sync. This project has two documentation audiences — humans and LLMs — and both must be updated.

## When to update docs

Update documentation whenever you:

- **Add a new sort** — update the family's `README.md` to list it. If it's a new family, create a new folder README.
- **Add a new strategy/trait implementation** — update the relevant `utils/` README (e.g. a new rotation goes in `utils/rotation/README.md`, a new gap sequence goes in `utils/shell_sequences/README.md`).
- **Add a new folder** — create a `README.md` in it explaining what it contains and why.
- **Change the registration system** — update `docs/registration.md` and `docs/llm/patterns.md`.
- **Change a trait interface** — update `docs/trait-system.md` and `docs/llm/patterns.md`.
- **Add or remove a crate** — update the root `README.md` (project structure + helper crates table) and `docs/architecture.md` (crate dependency graph).
- **Change the binary interface** — update the root `README.md` (binaries section).
- **Change key files that LLMs should index** — update `docs/llm/context-loading.md`.

## What to update where

| Change | Human docs to update | LLM docs to update |
|---|---|---|
| New sort in existing family | Family's `README.md` | Nothing (patterns unchanged) |
| New sort family (new folder) | `src/sorts/README.md`, new folder `README.md`, root `README.md` structure | `docs/llm/context-loading.md` if it introduces new key files |
| New rotation algorithm | `src/utils/rotation/README.md` | Nothing |
| New gap sequence | `src/utils/shell_sequences/README.md` | Nothing |
| New branching strategy | `src/utils/shell_branching/README.md` | Nothing |
| New trait | `docs/trait-system.md` | `docs/llm/patterns.md`, `docs/llm/context-loading.md` |
| Registration system change | `docs/registration.md`, `docs/adding-a-sort.md` | `docs/llm/patterns.md` |
| New crate in workspace | Root `README.md`, `docs/architecture.md` | `docs/llm/context-loading.md` |
| Changed logger API | `sort_logger/README.md`, `docs/trait-system.md`, `docs/adding-a-sort.md` | `docs/llm/patterns.md`, `docs/llm/context-loading.md` |

## Style guidelines

- **Folder READMEs** explain the concept first (what is a rotation? what is a gap sequence?), then list implementations with one-line descriptions.
- **`docs/` files** are for architecture and cross-cutting concerns. Don't duplicate folder-level details.
- **`docs/llm/` files** are practical and imperative — tell the LLM what to do, not background theory.
- Keep tables updated when adding rows (e.g. new rotation in the rotation README table).
- Don't add version history or changelogs to READMEs — that's what git is for.

## Checklist before finishing

Before you consider your work done, check:

- [ ] Did I add or modify a folder? Update/create its `README.md`.
- [ ] Did I add a new sort? Is it listed in the family's `README.md`?
- [ ] Did I change a trait or registration pattern? Update `docs/` and `docs/llm/`.
- [ ] Does the root `README.md` project structure still match reality?
- [ ] Would another LLM loading this project for the first time find accurate docs?
