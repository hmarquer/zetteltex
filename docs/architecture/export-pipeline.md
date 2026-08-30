# Export Pipeline

> **Map:** [Architecture Overview](overview.md) ← **Export Pipeline** → [Internals / cli export](../internals/zetteltex-cli.md) → [User-facing export guide](../guide/5-export.md)

The **export** pipeline converts notes and projects into Markdown for consumption outside ZettelTeX — primarily an [Obsidian](https://obsidian.md/) vault. It runs entirely in the CLI, using the parser to extract references and the database to enrich each note with metadata.

## Command surface

- `zetteltex export_markdown <note>` (or `<project>` with `--project`) — a single document.
- `zetteltex export_all_markdown [--notes] [--projects]` — everything in bulk.
- `zetteltex export_project <folder>` / `export_draft <input> <output>` — LaTeX-to-LaTeX expansions (not Markdown); see the subcommands below.

## Markdown flow

1. **Precondition**: sync is run first (`synchronize_notes` + `synchronize_projects`), so the database is fresh before anyone asks it questions.
2. For each target:
   - read the `.tex`, run `parse_note`/`parse_project_inclusions`;
   - query the database for **metadata** — labels, references, backlinks (`notes_referencing_note`), citations, projects that include the note;
   - write a `.md` file containing:
     - **YAML frontmatter**: `title`, `filename`, `created`, `last_edit_date`, `last_build_date_pdf/html`, `labels`, `references`, `backlinks`, `citations`, `projects`, `tags`;
     - **PDF embeds**: `![[<note>.pdf]]` (Obsidian syntax);
     - **Reference links** to sibling notes (`./<note>.md`);
     - **Tags** derived from the note's context.

3. Output directories come from config (`export.obsidian_vault` + subdirectories), e.g. `jabberwocky/latex/zettelkasten` for notes and `jabberwocky/latex/asignaturas` for projects by default.

## Obsidian-specific behavior

- Vault-style `[[...]]` wikilinks and `![[...]]` embeds are used so Obsidian renders and backlinks natively.
- If `render.pdf_output_dir` is inside the vault, PDF embeds resolve with correct relative paths (a tip restated in the [config reference](../reference/config-reference.md)).

## Subcommands that expand LaTeX

Two export commands work on LaTeX rather than Markdown:

- `export_project` — expands `\transclude[tag]{note}` into a standalone `.tex` per project folder.
- `export_draft` — expands `\ExecuteMetaData[file]{tag}` blocks for selective inclusion.

These are documented individually at [export_project](../reference/commands/export_project.md) and [export_draft](../reference/commands/export_draft.md).

---

## See Also

- Up: [Architecture Overview](overview.md) — pipeline list
- Down: [Internals / cli export](../internals/zetteltex-cli.md) — `export_note_markdown_file`, `export_project_markdown_file`, transclusion expansion
- Lateral: [Guide / Markdown Export](../guide/5-export.md) — user-facing vault setup