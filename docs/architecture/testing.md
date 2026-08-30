# Testing Strategy

> **Map:** [Architecture Overview](overview.md) ← **Testing Strategy** → [Internals / tests](../internals/functions.md)

ZettelTeX relies on a single, lean test layer: **86 integration tests** driven through the real `zetteltex` binary plus a handful of unit tests. There is no separate test crate.

## Where the tests live

| Location | Tests | What they exercise |
|---|---|---|
| `crates/zetteltex-cli/tests/cli_smoke.rs` | ~86 `#[test]` | End-to-end behavior of the compiled binary |
| `crates/zetteltex-cli/src/sync.rs` | 3 unit tests | `resolve_note_id` exact-match, `collect_tex_files` symlink skipping |
| `crates/zetteltex-cli/src/fuzzy.rs` | 6 unit tests | Preview extraction, popularity TSV parsing, terminal launchers |
| `crates/zetteltex-parser/src/lib.rs` | 3 unit tests | LaTeX comment handling, `\transclude` parsing |

The parser is exercised *indirectly* at scale through the integration tests (broken references, transclusions, renames that rewrite `\excref`, export with expansion).

## How a fixture workspace is built

`setup_workspace` (in `cli_smoke.rs`) creates a temp directory with:

- `notes/slipbox/`, `projects/`, `template/` (the minimal three-directory structure)
- `notes/documents.tex`
- minimal `note.tex` / `project.tex` templates
- a `zetteltex.toml` (`lang="es"`, `editor="code"`)

For isolated cases there is `setup_minimal_workspace`.

## Fake external tools

Rendering would normally shell out to `pdflatex`, `make4ht`, and `biber`. Tests instead install **fake tools**: small `sh` scripts, placed early in `PATH`, that log their arguments and exit 0. The fake `pdflatex` also **cats its input file**, letting tests assert on the exact LaTeX injected into a temporary render copy (e.g. the "Referenciado en" section).

## Things the tests do assert

- CLI basics: `--help` output, exit codes, invalid subcommand/name/format rejection.
- Sync + validation: broken `\excref`/`\ref`/`\transclude` detections, notes-only/projects-only scoping.
- CRUD: `newnote`/`newproject` and their database rows; `rename_note`/`remove_note` updating files and DB; rename scrubbing labels (including the interactive path).
- Export: Obsidian frontmatter, embeds, transclusion/`ExecuteMetaData` expansion.
- Render with fake tools: PDF pass counts with/without citations, HTML make4ht+biber, "Referenciado en" injection, a↔b circular pre-render, `render_all` batch, `render_updates` staleness (seeded by writing DB timestamps directly: a stale `1900-…` vs fresh `9999-…` build date).
- Failure modes: render without `pdflatex`/`biber`, note/project ambiguity without `--project`.
- Fuzzy, `edit`, `init_config`/i18n/babel.

Tests inspect SQLite directly (`rusqlite::Connection`) to assert on database rows after commands run — not just on stdout.

## What is NOT tested (a documented gap)

- **Real external tools**: fake `pdflatex`/`make4ht`/`biber` means real TeX engine behaviors (citations, cross-document references, math rendering) are covered only by manual verification.
- **Windows**: tests and cocurations assume a POSIX-ish toolchain (shell fake tools).
- **Large workspaces / performance**: no benchmarks or stress tests for the O(n) warm-ups in `render_all`.
- **`zetteltex-db` and `zetteltex-core`**: no crate-specific integration tests beyond what the CLI smoke tests exercise indirectly.

## Running them

```bash
cargo test -p zetteltex-cli
```

---

## See Also

- Up: [Architecture Overview](overview.md) — crate layout
- Down: [Internals / parser](../internals/zetteltex-parser.md) — what the parser-level tests cover
- Lateral: [CI workflow](../../.github/workflows/ci.yml) — how the suite runs in CI