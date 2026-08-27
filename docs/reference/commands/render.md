# `zetteltex render`

Compiles a single note or project into a PDF or HTML document.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] render <name> [OPTIONS]
```

---

## Arguments & Options

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `<name>` | string | Yes | — | Stem filename of the note or project to render. |
| `--project` | flag | No | `false` | Explicitly disambiguate target as a project. |
| `--format <pdf\|html>` | enum | No | `pdf` | Output format (`pdf` or `html`). |
| `--biber` | flag | No | `false` | Force execution of Biber bibliography engine. |

---

## PDF Render Execution Pipeline

When compiling to PDF (`--format pdf`), ZettelTeX orchestrates a deterministic multi-pass sequence:

1. **Backlink Source Verification**: Ensures that `.aux` files of notes linking to this note exist in the output directory so reverse hyperlinks resolve properly.
2. **Pass 1 (`pdflatex`)**: Emits initial `.aux` and `.bcf` files.
3. **Bibliography (`biber`)**: Triggered automatically if citations (`\cite{...}`) are present or `--biber` was passed.
4. **Pass 2 (`pdflatex`)**: Resolves labels, forward links, and cross-document references.
5. **Pass 3 (`pdflatex`)**: Run only when Biber was executed to eliminate BibLaTeX rerun notices.
6. **Timestamp Update**: Records `last_build_date_pdf` in `slipbox.db`.

The branching between Pass 1 and the Biber step depends on whether the document contains citations (or `--biber` was passed); Pass 3 then runs only if Biber executed.

---

## HTML Render Pipeline

When compiling to HTML (`--format html`):
1. Runs `make4ht` pass 1.
2. Invokes `biber` if citations are detected.
3. Runs `make4ht` pass 2.
4. Executes HTML post-processing (asset relocation).
5. Updates `last_build_date_html` in `slipbox.db`.

---

## Exit Codes

* **`0`**: Document compiled successfully.
* **`1`**: Compilation failure in `pdflatex`, `biber`, or `make4ht`.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Render note to PDF
zetteltex render compactness-in-metric

# Force Biber bibliography run
zetteltex render compactness-in-metric --biber

# Render as HTML
zetteltex render compactness-in-metric --format html

# Render project explicitly
zetteltex render topology-course --project
```

---

## See Also

* [`render_all`](render_all.md) — Render all documents in parallel.
* [`render_updates`](render_updates.md) — Incremental render for stale files.
* [`biber`](biber.md) — Run Biber independently.
