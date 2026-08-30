# Internals — Function & Code Reference

> **Map:** [Architecture Overview](../architecture/overview.md) → **Internals** → [Generated rustdoc](https://docs.rs)

This layer is a **curated index**, not a function dump. Each crate page gives you three things:

1. a link to that crate's generated rustdoc (`cargo doc --open --no-deps`);
2. a "where do I start reading?" file map for common tasks;
3. prose on the handful of functions that are load-bearing or non-obvious.

Signatures live in the code as rustdoc — pages here point at them rather than duplicating them, so they can't drift out of sync.

## Per-crate pages

| Crate | Page | Start reading here |
|---|---|---|
| `zetteltex-core` | [zetteltex-core.md](zetteltex-core.md) | `crates/zetteltex-core/src/lib.rs` — `WorkspacePaths`, `validate_component_name` |
| `zetteltex-db` | [zetteltex-db.md](zetteltex-db.md) | `crates/zetteltex-db/src/lib.rs` — `Database`, `init_database`, `migrate` |
| `zetteltex-parser` | [zetteltex-parser.md](zetteltex-parser.md) | `crates/zetteltex-parser/src/lib.rs` — 8 regexes, `parse_note`, `parse_project_inclusions` |
| `zetteltex-cli` | [zetteltex-cli.md](zetteltex-cli.md) | `crates/zetteltex-cli/src/main.rs` (dispatch), `cli.rs` (clap), then per-feature modules |

## Load-bearing functions worth reading in prose

Detailed discussion (with snippets) lives on the crate pages; the short list:

| Function                                  | Location                | Why it matters                                                               |
| ----------------------------------------- | ----------------------- | ---------------------------------------------------------------------------- |
| `WorkspacePaths::discover` / `validate`   | `core/src/lib.rs`       | Everything starts from validated paths; whole-command failures hang off this |
| `validate_component_name`                 | `core/src/lib.rs`       | The path-traversal guard on every user-supplied name                         |
| `needing_render_generic`                  | `db/src/lib.rs`         | The staleness criterion that drives `render_updates`                         |
| `parse_note` / `backslash` parity for `%` | `parser/src/lib.rs`     | Correct extraction of LaTeX commands from real-world notes                   |
| `parse_project_inclusions`                | `parser/src/lib.rs`     | Transclusion map; line-oriented, asymmetric with `parse_note`                |
| `resolve_note_or_project`                 | `cli/src/util.rs`       | Note-vs-project disambiguation, `--project` semantics                        |
| `resolve_note_id` (project sync)          | `cli/src/sync.rs`       | Strict transclusion check — sync makes this fatal                            |
| `needing_render_generic` SQL              | `db/src/lib.rs`         | NULL/edit-vs-build date comparison                                           |
| `RenderTarget::contains_citations`        | `cli/src/render/mod.rs` | Biber auto-detection via the real parser                                     |
| `ensure_backlink_sources`                 | `cli/src/render/pdf.rs` | mtime-based pre-render of referencing notes                                  |

## Staying in sync

These pages are hands-written complements to the source. When you refactor, update the crate page you touched and adjust the file-map tables. The generated rustdoc is the authoritative signature source.

---

## See Also

- Up: [Architecture Overview](../architecture/overview.md) — why the crates are split this way
- Down: [Generated rustdoc](https://docs.rs) — authoritative signatures
- Lateral: [Reference / commands](../reference/commands.md) — the user-facing surface these functions implement