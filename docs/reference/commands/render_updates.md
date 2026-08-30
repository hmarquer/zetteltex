# `zetteltex render_updates`
> **Map:** [Command Reference](../commands.md) → **`zetteltex render_updates`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Incrementally recompiles only the notes and projects whose source has changed since their last successful render.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] render_updates [OPTIONS]
```

---

## Options & Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--format <pdf\|html>` | enum | `pdf` | Output format (`pdf` or `html`). |
| `-j`, `--workers <N>` | integer | `4` | Number of parallel worker threads to spawn. |

---

## Behavior & Internal Workflow

1. **Synchronize indexes**:
   Runs `synchronize_notes` and `synchronize_projects` (protected by a SQLite write-lock retry) so the database reflects the latest edits on disk.
2. **Select stale items**:
   Queries the database for documents whose `last_edit_date` is newer than their `last_build_date_pdf` / `last_build_date_html`. Temporary render notes and files that no longer exist on disk are excluded.
3. **Warm-up backlink sources**:
   Builds an incoming-references index and pre-compiles any missing `.aux` dependencies for stale notes so cross-references resolve during concurrent builds.
4. **Parallel compilation**:
   Compiles the stale notes concurrently, then the stale projects, printing a plan summary (counts, workers, format, output directory).
5. **Update database timestamps**:
   Records `last_build_date_pdf` / `last_build_date_html` for every successfully compiled document.

> If nothing is stale, the command prints "No items pending render." and exits successfully.

---

## Exit Codes

* **`0`**: All stale documents rendered (or nothing was pending).
* **`1`**: One or more documents failed compilation.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Render only documents modified since their last build
zetteltex render_updates

# Use 6 parallel workers
zetteltex render_updates -j 6

# Update HTML output only
zetteltex render_updates --format html
```

---

## See Also

* [`render`](render.md) — Compile a single document.
* [`render_all`](render_all.md) — Compile all documents unconditionally.
* [`biber`](biber.md) — Run the bibliography processor manually.
