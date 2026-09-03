# Internals — `zetteltex-cli`

> **Map:** [Architecture Overview](../architecture/overview.md) → [Internals](functions.md) → **zetteltex-cli** → [Generated rustdoc](https://docs.rs/zetteltex_cli)

`zetteltex-cli` is the binary crate (`zetteltex`). It owns command definition (clap), dispatch, and all orchestration: sync, render, export, rename, maintenance, fuzzy search, and the editor/TUI integration. It is the **only** crate that talks to external tools (`pdflatex`, `make4ht`, `biber`, the user's editor).

## Module map

| Module | File | Role |
|---|---|---|
| Entry | `src/main.rs` | `main()`, `run_command` dispatch, fuzzy scripted/inline, PDF opener |
| CLI definition | `src/cli.rs` | `Cli` + `Commands` enum (clap derive), `OutputFormat` |
| Sync | `src/sync.rs` | `synchronize_notes`, `synchronize_projects`, `validate_references` |
| Export | `src/export.rs` | Markdown export, `export_project`, `export_draft` |
| Render | `src/render/mod.rs` | `RenderTarget`, orchestration, Biber, backlinks |
| Render engine | `src/render/engine.rs` | retry-on-lock SQLite helper |
| Render PDF | `src/render/pdf.rs` | pdflatex passes, backlink sources |
| Render HTML | `src/render/html.rs` | make4ht pass, HTML overrides, postprocess |
| Render progress | `src/render/progress.rs` | parallel progress bar |
| Watch | `src/watch.rs` | polling change detection, dispatch to render |
| Notes | `src/notes.rs` | CRUD notes/projects, list, edit |
| Workspace | `src/workspace.rs` | `init_workspace`, templates, `init_config` |
| Rename | `src/rename.rs` | `rename_note`, `remove_note`, label scrubbing |
| Maintenance | `src/maintenance.rs` | `clean`, `remove_duplicate_citations` |
| Fuzzy | `src/fuzzy.rs` | fuzzy index, config, popularity, launchers |
| UI | `src/ui.rs` | ratatui TUI for fuzzy search |
| HTML post | `src/html.rs` | postprocess HTML output (SVG, CSS, assets) |
| Util | `src/util.rs` | `resolve_note_or_project`, external runner, title extraction |
| LSP | `src/lsp.rs` | JSON-RPC 2.0 LSP server over stdio; `textDocument/completion` for `\excref`/`\exref`/`\exhyperref` |
| i18n | `src/i18n.rs` | re-export of core's `tr`/`set_lang` |

## Command dispatch (the literal bridge to the Reference layer)

Defined in `cli.rs:31` (enum `Commands`), dispatched in `main.rs:128` (`run_command`). `Init` runs **before** workspace discovery; everything else needs a valid workspace.

| Subcommand (Reference) | Fields | Dispatch → implements in |
|---|---|---|
| `init` | — | `main.rs:79` (early) / `:130` → `workspace.rs:84 init_workspace` |
| `init_config` | — | `workspace.rs:234 init_config_interactive` |
| `newnote` | `name` | `notes.rs:55 create_note` |
| `rename_note` | `name` | `rename.rs:55 rename_note` |
| `remove_note` | `name` | `rename.rs:208 remove_note` |
| `list_recent_files` | `n` (default 10) | `notes.rs:91 list_recent_files` |
| `list_unreferenced` | — | `notes.rs:111 list_unreferenced` |
| `rename_recent` | `n` (default 1) | `rename.rs:17 rename_recent` |
| `addtodocuments` | `name` | `notes.rs:134 add_to_documents` |
| `list_citations` | `name` | `notes.rs:173 list_citations` |
| `newproject` | `name` | `notes.rs:18 create_project` |
| `list_projects` | — | `notes.rs:221 list_projects_cmd` |
| `list_project_inclusions` | `project` | `notes.rs:242 list_project_inclusions_cmd` |
| `list_note_projects` | `note` | `notes.rs:293 list_note_projects_cmd` |
| `list_keywords` | `keyword?`, `--notes`/`--projects` | `notes.rs:336 list_keywords_cmd` |
| `export_project` | `folder`, `texfile?` | `export.rs:473 export_project` |
| `export_draft` | `input_file`, `output_file` | `export.rs:359 export_draft` |
| `export_markdown` | `note`, `--project` | via `resolve_note_or_project` → `export.rs:159 export_markdown` / `export.rs:265 export_project_markdown` |
| `export_all_markdown` | `--notes`/`--projects` | `export.rs:345 export_all_markdown` |
| `render` | `name`, `--project`, `--format`, `--biber` | via `resolve_note_or_project` → `render/mod.rs:67 render_note_cmd` / `:134 render_project_cmd` |
| `render_all` | `--format`, `-j`, `--notes`/`--projects` | `render/mod.rs:201 render_all_notes_cmd` / `:342 render_all_projects_cmd` |
| `render_updates` | `--format`, `-j` | `render/mod.rs:492 render_updates_cmd` |
| `watch` | `name?`, `--project`, `--format`, `-j`, `--poll` | `watch.rs:21 watch_cmd` → `render_note_cmd`/`render_project_cmd`/`render_updates_cmd` |
| `biber` | `name`, `--project`, `--folder` | via `resolve_note_or_project` → `render/mod.rs:876 run_biber_cmd` / `:896 run_biber_project_cmd` |
| `synchronize` | — | `sync.rs:71 synchronize_notes` + `:272 synchronize_projects` |
| `force_synchronize` | `--notes`/`--projects` | `sync.rs:71`/`:272` |
| `validate_references` | `--notes`/`--projects` | `sync.rs:163 validate_references` |
| `edit` | `name?`, `--project` | `notes.rs:195 edit_cmd` |
| `fuzzy` | `--inline`, `--action`, `--query`, … | `main.rs:406 fuzzy_cmd` |
| `clean` | — | `maintenance.rs:11 clean_cmd` |
| `remove_duplicate_citations` | — | `maintenance.rs:83 remove_duplicate_citations_cmd` |

The user-facing surface of this same table is [reference/commands.md](../reference/commands.md); this table is the implementation side.

## Load-bearing functions (prose)

### `resolve_note_or_project` — `util.rs:24`

The disambiguation gate for every note/project command. `--project` forces project; without it, a name that exists only as a project resolves to the project; a name that exists as **both** without `--project` is an error (never a silent guess); a name that matches neither is an error. All target-taking commands funnel through it, so the rule is defined once.

### `synchronize_notes` / `synchronize_projects` — `sync.rs:71` / `:272`

The two sync phases, both transaction-wrapped with a RAII `TransactionGuard` (`commit` on success, `rollback` on drop). Notes: purge temp-render notes → walk slipbox → `parse_note` → upsert note/labels/citations → `clear_links` → second pass resolving references into `link` rows. Projects: upsert project, recursively collect `.tex`s, `parse_project_inclusions`, `replace_project_inclusions` — with `resolve_note_id` (`sync.rs:357`) making a transclusion to a nonexistent note **fatal**. Both phases also detect keywords (via `util::extract_keywords_from_content` using the `[keywords] list` from config) and store them — with their exact `source_file` and `line` — using `replace_note_keywords`/`replace_project_keywords`; the export (`db.note_keywords`) and `list_keywords` (`db.list_note_keywords`/`list_project_keywords`) then read them back from the database. Full flow in [Sync Process](../architecture/sync-process.md).

### `validate_references` — `sync.rs:163`

Re-syncs if needed, then checks `missing_note`/`missing_label` for `\excref`/`\exhyperref` (`check_reference` `sync.rs:136`), internal `\ref` against file labels, project-local `\ref` across all project files (two passes), and `\transclude` → `missing_note`.

### `RenderTarget::contains_citations` — `render/mod.rs:52`

Decides whether **Biber** runs, using the *real parser* (`parse_note`) rather than a separate code path, so notes and projects can never disagree about citation detection. See [Render Pipeline](../architecture/render-pipeline.md).

### `render_updates_cmd` — `render/mod.rs:492`

The incremental-render orchestrator: re-sync → ask the database `notes_needing_render*`/`projects_needing_render*` (the `needing_render_generic` staleness SQL) → filter temp notes and vanished files → parallel-render only the stale ones → stamp build dates. `render_all_notes_cmd` (`:201`) is the "everything" sibling with an O(n) backlink warm-up before the parallel batch.

### `ensure_backlink_sources` — `render/pdf.rs:123`

Pre-renders referencing notes so `\externaldocument` backlinks resolve: a note is pre-rendered when its `.aux`/`.pdf` are missing or its `.tex` mtime is newer than its `.aux`. This mtime check intentionally bypasses the database timestamps. The "Referenciado en" section itself is injected into the temp copy by `inject_referenced_in_section` (`render/mod.rs:838`); HTML uses `inject_html_overrides` (`html.rs:121`, math `$$`, neutralized `\href`).

### `run_with_sqlite_lock_retry` — `render/engine.rs:3`

Wraps database operations during parallel renders with retry-on-lock, since parallel workers can transiently contend on `slipbox.db`.

## File map

| Task | Start reading at |
|---|---|
| Add/rename a subcommand | `cli.rs` (enum) → `main.rs` `run_command` → target module |
| Trace a render | `render/mod.rs` `render_*_cmd` → `render/pdf.rs` / `render/html.rs` |
| Trace a sync | `sync.rs` `synchronize_*` → parser → db calls |
| Extend export frontmatter | `export.rs` `push_frontmatter_*` |
| Extend the fuzzy TUI | `fuzzy.rs` (config/actions) + `ui.rs` (widgets) |

---

## See Also

- Up: [Architecture Overview](../architecture/overview.md) — crate graph and pipelines
- Down: [Generated rustdoc](https://docs.rs/zetteltex_cli) — signatures; [Testing Strategy](../architecture/testing.md) — the smoke suite that pins this behavior
- Lateral: [Reference / commands](../reference/commands.md) — the user-facing side of the dispatch table