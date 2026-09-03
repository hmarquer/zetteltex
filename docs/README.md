# ZettelTeX

## What is ZettelTeX

ZettelTeX is a command-line tool for managing a Zettelkasten knowledge base built on LaTeX. It handles note creation, cross-reference tracking, incremental PDF/HTML rendering, Markdown export, and fuzzy search — all from a single terminal interface.

If you maintain interconnected LaTeX notes — for coursework, research, or personal knowledge management — ZettelTeX automates the bookkeeping: keeping your database in sync, only recompiling what changed, searching notes instantly by name or content, and exporting to tools like Obsidian.

## Key Features

- **Incremental rendering** — Only recompiles notes whose source has changed. Supports PDF (via `pdflatex`/`biber`) and HTML (via `make4ht`).
- **Cross-reference tracking** — Extracts `\label`, `\ref`, `\cite`, and `\transclude` commands into a local SQLite database. Detects broken references and orphan notes.
- **Parallel rendering** — Renders multiple notes concurrently with configurable worker count.
- **Markdown export** — Converts notes and projects to Markdown with frontmatter, PDF embeds, and inter-note links.
- **Fuzzy search** — Built-in TUI for quick note lookup, cross-reference insertion, and clipboard operations.
- **VS Code extension** — Contextual autocomplete of notes and labels plus link snippets (`exc`/`exr`/`exh`) inside `\excref`, `\exref`, and `\exhyperref` (see [VS Code extension](../editors/vscode/README.md)).
- **Bilingual interface** — Spanish and English, configurable via `zetteltex.toml`. Runtime messages follow the configured language, but the built-in `--help`/usage text is **English only** (it is generated statically at compile time).

## Installation

### From source

```bash
cargo install --path crates/zetteltex-cli --force
```

### From GitHub Releases

Download the prebuilt binary for your platform from the [Releases](https://github.com/hmarquer/zetteltex/releases) page, then:

```bash
# Linux / macOS
chmod +x zetteltex
sudo mv zetteltex /usr/local/bin/

# Windows: place zetteltex.exe in a directory listed in your PATH
```

## Getting Started

To set up your first workspace and start writing notes, follow the [User Guide](guide/0-getting-started.md) — it walks you through prerequisites, workspace creation, configuration, note types, linking, and rendering.

## Documentation Map

Read the layers in order — each one zooms in on the previous. Every page ends with a **See Also** block that links one layer up (concept) and one layer down (implementation).

| Layer | Reading order | Audience |
|---|---|---|
| [**1. User Guide**](#user-guide) | `guide/0` → `guide/7` | End users |
| [**2. Command Reference**](#reference) | [Reference index](reference/README.md) | End users |
| [**3. Architecture**](#architecture) | [Architecture overview](architecture/overview.md) → satellites | Contributors |
| [**4. Internals**](#internals) | [function map](internals/functions.md) → per-crate pages | Contributors |

### User Guide
A step-by-step linear guide for end users:

1. [**0. Getting Started**](guide/0-getting-started.md) — Prerequisites, workspace initialization, and configuration.
2. [**1. Notes and Projects**](guide/1-notes-and-projects.md) — Atomic notes, project documents, and editing.
3. [**2. Linking Notes**](guide/2-linking.md) — Cross-references (`\excref`, `\exref`, `\exhyperref`), transclusions, and synchronization.
4. [**3. Rendering**](guide/3-rendering.md) — PDF/HTML compilation, Biber integration, and incremental builds.
5. [**4. Fuzzy Search**](guide/4-fuzzy-search.md) — Terminal UI, keyboard shortcuts, and scripted actions.
6. [**5. Markdown Export**](guide/5-export.md) — Obsidian vault integration, YAML frontmatter, and PDF embeds.
7. [**6. Daily Workflow**](guide/6-daily-workflow.md) — End-to-end daily routine and command cheat sheet.
8. [**7. Troubleshooting**](guide/7-troubleshooting.md) — Common error resolution, diagnostics, and recovery.

The [VS Code extension](guide/2-linking.md#vs-code-extension) (autocomplete + snippets) is covered under **Linking Notes**.

### Reference

| Section | Description | Audience |
|---|---|---|
| [**Command Reference**](reference/commands.md) | Complete list of all commands, flags, global options, exit codes, and configuration fields. | End users |
| [**Configuration Reference**](reference/config-reference.md) | Full specification of `zetteltex.toml` | End users |
| [**Global Options**](reference/global-options.md) | Binary-wide flags and environment variables | End users |
| [**Exit Codes**](reference/exit-codes.md) | Process return codes and scripting guidance | End users / CI |

### Architecture

| Section | Description | Audience |
|---|---|---|
| [**Overview**](architecture/overview.md) | Crate diagram and responsibilities — start here | Contributors |
| [**Workspace Model**](architecture/workspace-model.md) | On-disk layout, discovery, note vs project | Contributors |
| [**Data Model**](architecture/data-model.md) | `slipbox.db` schema, entities, migrations | Contributors |
| [**Sync Process**](architecture/sync-process.md) | How metadata stays in sync | Contributors |
| [**Render Pipeline**](architecture/render-pipeline.md) | PDF/HTML compilation at the system level | Contributors |
| [**Export Pipeline**](architecture/export-pipeline.md) | Markdown/Obsidian export | Contributors |
| [**Testing Strategy**](architecture/testing.md) | What the tests cover and what they don't | Contributors |

### Internals

| Section | Description | Audience |
|---|---|---|
| [**Function Map**](internals/functions.md) | Index: file maps + load-bearing functions per crate | Contributors |
| [**zetteltex-core**](internals/zetteltex-core.md) | Workspace discovery and validation | Contributors |
| [**zetteltex-db**](internals/zetteltex-db.md) | Schema, migrations, staleness SQL | Contributors |
| [**zetteltex-parser**](internals/zetteltex-parser.md) | LaTeX command extraction | Contributors |
| [**zetteltex-cli**](internals/zetteltex-cli.md) | Command dispatch and pipelines | Contributors |

Generated rustdoc (via `cargo doc --open --no-deps`) is the authoritative signature reference for all four crates.


## Contributing

ZettelTeX is a Rust workspace with four crates:

| Crate | Responsibility |
|---|---|
| `zetteltex-core` | Workspace discovery and path validation |
| `zetteltex-db` | SQLite persistence and migrations |
| `zetteltex-parser` | LaTeX parsing (labels, citations, references) |
| `zetteltex-cli` | Command dispatch, rendering, export, TUI |

To run the test suite:

```bash
cargo test -p zetteltex-cli
```

For architecture details, see the [Architecture](architecture/overview.md) section.

## License

MIT
