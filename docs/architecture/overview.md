# Architecture Overview

> **Map:** [Repo README](../README.md) → [User Guide](../guide/0-getting-started.md) → [Command Reference](../reference/commands.md) → **Architecture** → [Internals](../internals/functions.md) → [Generated rustdoc](https://docs.rs)

ZettelTeX is a Rust **workspace** of four crates with a strict dependency direction. This page is the hub for contributors: it explains why the system is split the way it is and points to one satellite page per concern.

## Crate dependency diagram

```
                        ┌─────────────────────┐
                        │      zetteltex-cli    │   binary `zetteltex`
                        └──────────┬──────────┘
                 ┌─────────────────┼─────────────────┐
                 │                 │                 │
                 ▼                 ▼                 ▼
        ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
        │ zetteltex-core│  │  zetteltex-db │  │ zetteltex-parser│
        └──────┬───────┘  └──────────────┘  └──────────────┘
               │                 │
               └───────►─────────┘   db → core (i18n `tr!` macro)
```

Dependencies are single-direction: `zetteltex-cli` (the binary) uses the other three; `zetteltex-db` depends on `zetteltex-core` only for the i18n `tr!` macro; `zetteltex-parser` has no internal dependencies.

## Crate responsibilities

| Crate | Responsibility | Why the boundary is here |
|---|---|---|
| [`zetteltex-core`](../internals/zetteltex-core.md) | Workspace layout (`notes/slipbox`, `projects`, `template`), discovery and path validation, runtime i18n | Tiny, dependency-free; anything value-neutral that other crates share lives here |
| [`zetteltex-db`](../internals/zetteltex-db.md) | The SQLite `slipbox.db` schema, migrations, and ~50 query/mutation methods | All persistence in one place so the CLI never touches SQL directly and the schema can evolve centrally |
| [`zetteltex-parser`](../internals/zetteltex-parser.md) | Regex-based extraction of `\label`, `\currentdoc`, `\cite`, `\ref`, `\excref`, `\exhyperref`, `\exref`, `\transclude` from `.tex` files | Parsing is self-contained and testable in isolation from the CLI and database |
| [`zetteltex-cli`](../internals/zetteltex-cli.md) | Command dispatch (clap), sync, render, export, fuzzy TUI | The only consumer; orchestration and external-tool invocation (`pdflatex`, `make4ht`, `biber`) never leak into the other crates |

## Reading path

A contributor usually starts in one of these, depending on what they touch:

| You want to | Start at |
|---|---|
| Understand the on-disk workspace | [Workspace Model](workspace-model.md) |
| Understand the SQLite database | [Data Model](data-model.md) |
| Understand how metadata stays in sync | [Sync Process](sync-process.md) |
| Understand how PDF/HTML output is produced | [Render Pipeline](render-pipeline.md) |
| Understand Obsidian/Markdown export | [Export Pipeline](export-pipeline.md) |
| Understand what the tests cover | [Testing Strategy](testing.md) |

For code-level detail (which function backs which command, the load-bearing functions), continue to [Internals](../internals/functions.md).

---

## See Also

- Up: [Contributing](../README.md#contributing) — crate table and test command in the repo README
- Down: [Internals / function map](../internals/functions.md) — where each crate's code lives
- Lateral: [Command Reference](../reference/commands.md) — the user-facing surface this architecture supports