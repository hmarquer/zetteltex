# Render Pipeline

> **Map:** [Architecture Overview](overview.md) ← **Render Pipeline** → [Internals / cli render](../internals/zetteltex-cli.md) → [User-facing render command](../reference/commands/render.md)

The **render** pipeline turns a note or project into a PDF (via `pdflatex`) or HTML (via `make4ht`, the tex4ht toolchain). It is orchestrated entirely by the CLI; the parser and database play supporting roles.

## Input resolution

`resolve_note_or_project` (in the CLI) decides whether the given name is a note, a project, or ambiguous:

- `--project` forces project resolution.
- A name existing both as a note and a project **without** `--project` is an error (never a silent guess).
- Otherwise the existing target wins.

Both note and project render paths funnel through a single `RenderTarget` enum (`Note(name)` | `Project(name)`), so note/project differences never fork the shared orchestration. See [data-model.md](data-model.md) and the [name-disambiguation rule](../reference/commands.md#note-vs-project-resolution).

## Role of the parser

The parser decides **two things** that shape the pipeline:

1. **Biber**: `RenderTarget::contains_citations` runs the real `parse_note` on the target source. If any citation is found, Biber is interleaved between compiler passes — notes and projects can never diverge on this decision.
2. **"Referenciado en" (referenced-in) backlinks**: before compiling a note, the CLI scans the workspace for notes that reference it (using the parser's `references` output) and injects a `\section*{Referenciado en}` listing them, so the PDF/HTML shows who cites the note. This scan is done against disk via the parser, not the database.

## PDF pipeline (`render.pdf`)

```
pdflatex (pass 1)
   ├─ no citations ──► pdflatex (pass 2)            → 2 passes total
   └─ citations ─────► biber ──► pdflatex (final)   → 3 passes total
```

- Notes render from a **temporary copy** with the "Referenciado en" section injected; projects render their primary `<name>/<name>.tex` directly.
- Before the main run, **referencing notes are pre-rendered** (`ensure_backlink_sources`) if their `.aux`/`.pdf` are missing or their `.tex` mtime is newer than the `.aux` — this is what makes `\externaldocument` backlinks resolve. This is an mtime-based check, independent of the database.
- The engine is `pdflatex -interaction=nonstopmode` with job/project naming and `-output-directory` set from config (`render.pdf_output_dir`, default `pdf`); `--shell-escape` is used when configured.

## HTML pipeline (`render.html`)

```
make4ht -f html5+svg (pass 1)
   ├─ no citations ──► done
   └─ citations ─────► biber ──► make4ht (pass 2)
```

- Notes render from a temporary copy with "Referenciado en" plus HTML overrides that map display math to `$$` and neutralize `\href`/`\hyperref` for the web.
- After make4ht, `postprocess_html_output` scales SVG math, copies fonts/assets, rewrites asset paths, and applies CSS.
- Output goes to `render.html_output_dir` (default `html`).

## Batch renders

- `render_all` / `render_updates` build an **index of incoming references once** (O(n) warm-up), then run notes in parallel across `workers` threads with a single-line progress bar.
- `run_with_sqlite_lock_retry` wraps database writes, since parallel renders can transiently contend on `slipbox.db`.

## Staleness (`render_updates`)

`render_updates_cmd` first re-synchronizes, then asks the database which items need building. The criterion lives in the db crate (`needing_render_generic`, see [data-model.md](data-model.md)):

```
last_build_date_x IS NULL OR last_edit_date IS NULL OR last_edit_date > last_build_date_x
```

that is: never built, or edited more recently than the last build. These are database timestamps (edit time captured at sync; build time stamped after a successful compile), not filesystem mtimes — except for backlink pre-rendering, which uses mtimes directly.

## Exit codes and user-facing detail

Per-command behavior (flags like `--format`, `--biber`, `-j/--workers`) is documented in the [render command reference](../reference/commands/render.md) and [render_updates](../reference/commands/render_updates.md).

---

## See Also

- Up: [Architecture Overview](overview.md) — pipeline list
- Down: [Internals / cli render](../internals/zetteltex-cli.md) — `render_*_cmd`, `render_pdf`, `render_html_single_pass`, staleness helpers
- Lateral: [Reference / render](../reference/commands/render.md) — user-facing flags and examples