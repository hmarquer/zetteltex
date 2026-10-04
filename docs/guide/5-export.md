# Export
> **Map:** [Guide](0-getting-started.md) → **Export** → [Daily Workflow](6-daily-workflow.md)

ZettelTeX can export your LaTeX notes and projects in two ways:

* **Markdown** — rich YAML frontmatter, backlinks, and PDF embeds, for Markdown-based tools like [Obsidian](https://obsidian.md).
* **Standalone `.tex`** — a single self-contained LaTeX file with every `\transclude` expanded and the engine templates inlined, independent of the notes base.

## Configuration

Export settings are configured in `zetteltex.toml` under the `[export]` section:

```toml
[export]
# Path to your Obsidian vault (absolute or relative to workspace root)
obsidian_vault = "vault"

# Subdirectory for notes inside the vault
notes_subdir = "notes"

# Subdirectory for projects inside the vault
projects_subdir = "projects"
```

If `obsidian_vault` is omitted, ZettelTeX defaults to `<workspace-root>/jabberwocky`.

> **Tip for Obsidian users:** Set `pdf_output_dir` in `[render]` to a directory inside your vault (e.g. `vault/pdf`). When notes are exported, ZettelTeX inserts `![[note.pdf]]` embeds, allowing Obsidian to display the compiled PDF preview seamlessly next to the note metadata.

## Export Commands

### Export a single note or project

```bash
zetteltex export_markdown compactness-in-metric
```

Automatically detects whether the name corresponds to a note or a project, synchronizes metadata, and writes the Markdown file to the appropriate subdirectory.

If both a note and a project share the same name, specify `--project` to export the project:

```bash
zetteltex export_markdown topology --project
```

### Export all notes and projects

```bash
zetteltex export_all_markdown
```

Exports every note to `notes_subdir` and every project to `projects_subdir` in a single pass after synchronizing the database.

You can restrict the scope with flags:

```bash
# Export only notes
zetteltex export_all_markdown --notes

# Export only projects
zetteltex export_all_markdown --projects
```

## Generated Markdown Structure

### Note Markdown

For each note, ZettelTeX generates a `.md` file containing:

1. **YAML Frontmatter**:
   ```yaml
   ---
   title: "Compactness in Metric Spaces"
   filename: "compactness-in-metric"
   created: "2026-08-20T10:00:00Z"
   last_edit_date: "2026-08-26T18:30:00Z"
   last_build_date_pdf: "2026-08-26T18:35:00Z"
   last_build_date_html: null
   labels:
     - defn:compactness
     - thm:heine-borel
   references:
     - metric-spaces
     - open-covers
   backlinks:
     - topology-summary
     - analysis-notes
   citations:
     - rudin1976principles
   projects:
     - analysis-course
   tags:
     - analysis-course/chapter1
   ---
   ```

2. **PDF Embed**:
   ```markdown
   [[compactness-in-metric.pdf]]
   ![[compactness-in-metric.pdf]]
   ```

3. **Outgoing References**:
   ```markdown
   ## Referencias
   - [metric-spaces](./metric-spaces.md)
   - [open-covers](./open-covers.md)
   ```

4. **Keyword Tags**: Detected in the source (per line, as a substring — e.g. `TODO:`, `DEMOSTRACION`) using the `[keywords] list` from `zetteltex.toml`. They are stored in `slipbox.db` during synchronization and rendered as `#KEYWORD text` lines. Use [`zetteltex list_keywords`](../reference/commands/list_keywords.md) to browse which notes/projects carry each keyword. See the [Configuration Reference](../reference/config-reference.md).

### Project Markdown

For projects, the generated Markdown includes:
- YAML frontmatter with title, project name, timestamps, tags, and all included note names (`inclusions`).
- PDF embed linking to the compiled project PDF.
- A grouped listing of included notes organized by the subfiles (`\transclude`) where they appear.

## Additional Export Utilities

### Export a Standalone `.tex` File

```bash
zetteltex export my-project --output releases/my-project.tex
```

Generates a single, self-contained `.tex` file from a note or a project:

* The engine templates (`ztxbase.sty`, `style.sty`) and the class (`texnote.cls` / `texbook.cls`) are **inlined** into the preamble, so the file compiles anywhere with a TeX installation — no dependency on the notes base.
* Every `\transclude[tag]{note}` is **expanded recursively** into the body.
* Only the **citations actually used** are embedded (from `bibliography.bib`), and with no citations the bibliography is dropped entirely.
* `\exref`, `\excref` and `\exhyperref` keep linking to notes present in the file; references to notes that are not included print the note's name instead.

For a note and a project sharing a name, pass `--project` to disambiguate:

```bash
zetteltex export topology --project --output builds/topology.tex
```

The output path is resolved against the workspace root unless it is absolute.

The file compiles anywhere with a TeX installation: `pdflatex export.tex`; if the document has citations, run `biber export` and then `pdflatex` twice more (references and labels reset on successive passes).

## Cleaning Up Orphan Exports

When you rename or delete notes and projects, previous export artifacts may remain in your vault. Use `clean` to remove orphan `.md` and `.pdf` files that no longer exist in the database:

```bash
zetteltex clean
```

This scans your export directories and removes any generated `.pdf` or `.md` files without a matching record in `slipbox.db`.

## Next step

Review the recommended [Daily Workflow](6-daily-workflow.md) to integrate all these commands into your routine.

## See Also

* [Reference / `export_markdown`](../reference/commands/export_markdown.md) — command syntax.
* [Reference / `export`](../reference/commands/export.md) — standalone `.tex` export.
* [Export Pipeline](../architecture/export-pipeline.md) — how exports are generated.
