# Documentation maintenance

The docs have two audiences, people and assistants, and both must stay in
step with the code. Prose follows `/home/omer/Documents/dev/skillz/STYLE.md`:
ASCII punctuation only, no em dash, no Unicode arrow, no emoji, no
underscore in prose, a file top overview in every source file, a README in
every folder, comments that describe the code as it is and never as a diff.

Check added lines with:

```
python3 /home/omer/Documents/dev/skillz/scripts/check_style.py --added-only <paths>
```

## What to update where

| change | update |
|---|---|
| new slot filler | the axis folder README (`crates/pivots/README.md` and so on) and the new crate's README |
| new family | `crates/families/README.md`, the crate README, the catalog table in the root `README.md`, the golden count |
| new first class kind | `docs/registration.md` (the wrapper table and the kind recipe), `docs/architecture.md` (consumers), root `README.md` |
| a role trait changed | `docs/trait-system.md`, `docs/writing-components.md` |
| the logger API changed | `sort_logger/README.md`, `docs/trait-system.md`, `docs/writing-components.md` |
| the query grammar or `.spec` keywords changed | `docs/registration.md`, `docs/writing-components.md` |
| the build script changed | `docs/architecture.md`, `spec_catalog/README.md` |
| a crate added or removed | root `README.md` layout, `docs/architecture.md` graph, the parent folder README |
| a binary or flag changed | root `README.md` running table, `src/bin/README.md` |
| a plan started, finished or abandoned | its file in `docs/implementation_plans/` and the status list in `docs/todo.md` |
| key files for assistants moved | `docs/llm/context-loading.md` |

## Style of each document type

- Folder READMEs say what the folder represents, then one line per subfolder
  or file worth naming. Minimal, not a tour.
- `docs/` files cover architecture and cross cutting concerns and do not
  repeat folder level detail.
- `docs/llm/` files are imperative. Tell the assistant what to do.
- `docs/todo.md` is the one place for status. Vocabulary docs define
  concepts without status words.
- No changelogs in READMEs or comments. History lives in git.

## Checklist before finishing

- Did I add or move a folder or crate? Its README exists and the parent
  README lists it.
- Did I change a trait, a keyword or the grammar? The docs above reflect it.
- Did the catalog change? The root README table and the golden counts agree.
- Did I finish or start planned work? `docs/todo.md` says so.
- Would an assistant loading the project cold find accurate instructions?
