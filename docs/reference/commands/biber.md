# `zetteltex biber`

Manually runs the Biber bibliography processor on a specific note or project.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] biber <name> [folder] [--project]
```

---

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<name>` | string | Yes | Name of the note or project to process. |
| `[folder]` | string | No | Output directory where `.bcf` and `.bbl` files reside (defaults to the rendered PDF output directory). |

---

## Options & Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--project` | flag | `false` | Treat `<name>` as a project when it matches both a note and a project. |

---

## Behavior & Internal Workflow

1. **Resolve target**:
   The name is resolved as a note or a project. If a note and a project share the same name, `--project` disambiguates; otherwise the single match is used automatically.
2. **Determine output directory**:
   Uses `[folder]` if provided, otherwise the configured render output directory. The directory is created if missing and canonicalized.
3. **Run Biber**:
   Invokes `biber --output-directory=<dir> <name>` from the corresponding source directory (`notes/slipbox/` for notes, `projects/<name>/` for projects). This is typically needed after a first `pdflatex` pass has generated the `.bcf` control file.

> ZettelTeX normally invokes Biber automatically during `render` when citations are detected. This command is for manual or forced runs.

---

## Exit Codes

* **`0`**: Biber completed successfully.
* **`1`**: Biber is missing from `PATH` or returned a non-zero exit code.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Run Biber for a note, writing output to the default directory
zetteltex biber compactness

# Run Biber for a project
zetteltex biber analysis-course --project

# Specify a custom output directory
zetteltex biber compactness build/bib
```

---

## See Also

* [`render`](render.md) — Automatic compilation including Biber detection.
* [`render_updates`](render_updates.md) — Incremental recompilation.
