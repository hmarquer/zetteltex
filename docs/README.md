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
- **Bilingual interface** — Spanish and English, configurable via `zetteltex.toml`.

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

## Documentation

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

### Reference & Architecture

| Section | Description | Audience |
|---|---|---|
| [**Command Reference**](reference/commands.md) | Complete list of all commands, flags, global options, exit codes, and configuration fields. | End users |
| [**Architecture**](architecture/overview.md) | Internal design: crate structure, workspace model, data model, sync/render/export pipelines, testing strategy. | Contributors |
| [**Code Reference**](internals/functions.md) | Function signatures and responsibilities organized by crate. | Contributors |

The [Spanish documentation](es/README.md) is also available.


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
