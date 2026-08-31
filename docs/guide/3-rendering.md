# Rendering
> **Map:** [Guide](0-getting-started.md) → **Rendering** → [Fuzzy Search](4-fuzzy-search.md)

After synchronizing, you can render notes and projects to PDF or HTML.

## Render a single document

```bash
zetteltex render compactness-in-metric
```

Runs two passes of `pdflatex` to resolve cross-references and produces a PDF in the configured `pdf_output_dir`. If the name matches both a note and a project, ZettelTeX asks you to use `--project` to disambiguate.

### Bibliography

Biber runs automatically when the database detects that the note has `\cite{...}` commands. You can force it with `--biber` even if no citations are detected:

```bash
zetteltex render compactness-in-metric --biber
```

This forces biber between the two `pdflatex` passes, even if no citations were detected.

### HTML output

```bash
zetteltex render compactness-in-metric --format html
```

Uses `make4ht` instead of `pdflatex` and produces output in the configured `html_output_dir`.

## Render all documents

```bash
zetteltex render_all
```

Renders every note and project in the workspace with parallel workers. Use `-j` to control concurrency:

```bash
zetteltex render_all -j 4
```

### Render only notes or projects

```bash
zetteltex render_all --notes-only
zetteltex render_all --projects-only
```

## Render only stale items

```bash
zetteltex render_updates
```

Synchronizes metadata first, then renders only notes and projects whose `.tex` source has changed since the last render. This is the fastest way to update your output after editing.

## Watch and auto-recompile

```bash
zetteltex watch
```

Starts a loop that watches every LaTeX file in the workspace and automatically recompiles the affected documents whenever you save. Pass a name to watch a single note or project:

```bash
zetteltex watch anillo            # watch the note, recompile on change
zetteltex watch --project libro   # watch a project
```

Use `--format html` to output HTML, `-j` to control concurrency in whole-workspace mode, and `--poll <ms>` to tune how often it checks for changes. It renders the target once when it starts, then recompiles on every change until you press `Ctrl-C`.

## Next step

Use [Fuzzy Search](4-fuzzy-search.md) to quickly find and act on notes without leaving the terminal.

## See Also

* [Reference / `render`](../reference/commands/render.md) — flags and pipelines.
* [Render Pipeline](../architecture/render-pipeline.md) — the system-level view.
