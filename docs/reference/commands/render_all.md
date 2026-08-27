# `zetteltex render_all`

Compiles all notes and projects in the workspace in parallel with configurable worker threads.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] render_all [OPTIONS]
```

---

## Options & Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--format <pdf\|html>` | enum | `pdf` | Output format (`pdf` or `html`). |
| `-j`, `--workers <N>` | integer | `4` | Number of parallel worker threads to spawn. |
| `--notes-only` | flag | `false` | Compile only atomic notes in `notes/slipbox/`. |
| `--projects-only` | flag | `false` | Compile only projects in `projects/`. |

---

## Behavior & Internal Workflow

1. **Inverted Index Warm-Up ($O(1)$ lookup)**:
   Before spawning worker threads, ZettelTeX constructs an in-memory inverted index of incoming cross-references across the workspace. It pre-compiles any missing `.aux` dependencies so that all inter-document references resolve during concurrent builds without race conditions.
2. **Parallel Compilation Pool**:
   Spawns a thread pool of size $N$ (configured by `-j` / `--workers`).
3. **Progress Bar & Error Reporting**:
   Displays a progress indicator and summarizes any compilation failures at the end of the batch.
4. **Database Timestamps**:
   Updates `last_build_date_pdf` or `last_build_date_html` in `slipbox.db` for every successfully rendered document.

---

## Exit Codes

* **`0`**: All documents rendered without errors.
* **`1`**: One or more documents failed compilation.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Render all documents to PDF with default 4 workers
zetteltex render_all

# Render all documents using 8 parallel threads
zetteltex render_all -j 8

# Render only notes to HTML
zetteltex render_all --notes-only --format html
```

---

## See Also

* [`render_updates`](render_updates.md) — Fast incremental render.
* [`render`](render.md) — Single document render.
