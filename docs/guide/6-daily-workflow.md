# Daily Workflow
> **Map:** [Guide](0-getting-started.md) → **Daily Workflow** → [Troubleshooting](7-troubleshooting.md)

This guide outlines a recommended daily routine for capturing, linking, validating, and publishing notes with ZettelTeX.

## The Core Loop

A typical note-taking session follows these steps:

1. **Create Note** → create an atomic note.
2. **Edit & Link** → write content and add cross-references.
3. **Synchronize** → update the database.
4. **Render PDF/HTML** → produce compiled output.

### 1. Capture a new idea

Create an atomic note:

```bash
zetteltex newnote metric-compactness
```

This creates `notes/slipbox/metric-compactness.tex` from your template and registers it in the database and `notes/documents.tex`.

### 2. Write and link

Open the note in your configured editor:

```bash
zetteltex edit metric-compactness
```

While writing, connect your note to related concepts using:
- `\excref{open-covers}` — for automatic typed references.
- `\exhyperref{topology-basics}{topological spaces}` — for descriptive hyperlinks.
- `\cite{author2020}` — for bibliography citations.

> **Tip:** Use `zetteltex fuzzy` (or `Ctrl+H` / `Ctrl+R` in the TUI) to search your existing notes and copy the exact LaTeX reference macro directly into your clipboard.

### 3. Synchronize metadata

Update the local database with all new labels, citations, and links:

```bash
zetteltex synchronize
```

### 4. Compile and verify

Render the note to PDF:

```bash
zetteltex render metric-compactness
```

If the note includes citations, ZettelTeX automatically invokes `biber` between compiler passes.

---

## Working with Projects

When composing a larger document (e.g., lecture notes, a paper, or a thesis chapter):

1. **Create or open the project**:
   ```bash
   zetteltex newproject analysis-course
   zetteltex edit analysis-course
   ```

2. **Transclude notes**:
   Pull in relevant atomic notes without duplicating text:
   ```latex
   \transclude{metric-compactness}
   ```

3. **Render project**:
   ```bash
   zetteltex render analysis-course
   ```

---

## Session Wrap-Up: Quality & Publishing

Before finishing a work session, run a fast verification and update:

1. **Validate reference integrity**:
   ```bash
   zetteltex synchronize
   zetteltex validate_references
   ```
   Ensure no broken `\excref` or missing `\transclude` tags remain.

2. **Incremental build of changed documents**:
   ```bash
   zetteltex render_updates -j 4
   ```
   Only recompiles documents modified since their last build date.

3. **Export to Obsidian / Markdown**:
   ```bash
   zetteltex export_all_markdown
   ```
   Refreshes frontmatter, PDF embeds, and backlinks in your knowledge vault.

4. **Maintenance (as needed)**:
   ```bash
   zetteltex clean
   zetteltex remove_duplicate_citations
   ```

---

## Cheat Sheet

| Task | Command |
|---|---|
| Create note | `zetteltex newnote <name>` |
| Create project | `zetteltex newproject <name>` |
| Open in editor | `zetteltex edit [<name>]` |
| Fuzzy search / TUI | `zetteltex fuzzy` |
| Sync database | `zetteltex synchronize` |
| Check broken links | `zetteltex validate_references` |
| Fast incremental build | `zetteltex render_updates` |
| Build single doc | `zetteltex render <name> [--format html] [--biber]` |
| Build all docs | `zetteltex render_all [-j <workers>]` |
| Export to Obsidian | `zetteltex export_all_markdown` |
| Clean orphan files | `zetteltex clean` |

## Next step

If you encounter unexpected behavior or errors, see the [Troubleshooting](7-troubleshooting.md) guide.

## See Also

* [Command Reference](../reference/commands.md) — full syntax for every command used here.
* [Architecture Overview](../architecture/overview.md) — how the pipelines fit together.
