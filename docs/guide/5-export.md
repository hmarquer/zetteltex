# Markdown Export
> **Map:** [Guide](0-getting-started.md) → **Markdown Export** → [Daily Workflow](6-daily-workflow.md)

ZettelTeX can export your LaTeX notes and projects into Markdown files with rich YAML frontmatter, backlinks, and PDF embeds. This makes your knowledge base directly accessible in Markdown-based tools like [Obsidian](https://obsidian.md).

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

4. **Keyword Tags**: Detected in the source (per line, as a substring — e.g. `TODO:`, `DEMOSTRACION`) using the `[keywords] list` from `zetteltex.toml`. They are stored in `slipbox.db` during synchronization and rendered as `#KEYWORD text` lines. See the [Configuration Reference](../reference/config-reference.md).

### Project Markdown

For projects, the generated Markdown includes:
- YAML frontmatter with title, project name, timestamps, tags, and all included note names (`inclusions`).
- PDF embed linking to the compiled project PDF.
- A grouped listing of included notes organized by the subfiles (`\transclude`) where they appear.

## Additional Export Utilities

### Export a Project Bundle

```bash
zetteltex export_project my-project
```

Bundles a project folder and its primary `.tex` file into an export destination. You can optionally specify a custom main file with `--texfile`:

```bash
zetteltex export_project my-project --texfile custom_main.tex
```

### Export Draft with Metadata Expansion

```bash
zetteltex export_draft draft_input.tex draft_output.tex
```

Processes an input `.tex` document containing `\ExecuteMetaData[file]{tag}` statements, expands each tagged section inline from the target note files, and writes the assembled draft.

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
* [Export Pipeline](../architecture/export-pipeline.md) — how Markdown is generated.
