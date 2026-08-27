# Command Reference

Complete index of all subcommands available in the `zetteltex` CLI. Each command links to a dedicated page with full details.

---

## Command Invocation Pattern

```bash
zetteltex [--workspace-root <PATH>] <COMMAND> [ARGUMENTS] [OPTIONS]
```

* For global options such as `--workspace-root`, see [Global Options](global-options.md).
* For process return codes, see [Exit Codes](exit-codes.md).
* For configuration file options, see [Configuration Reference](config-reference.md).

---

## Name Disambiguation Rule

Several commands (`render`, `export_markdown`, `biber`) accept a `<name>` argument that can refer to either an atomic note in `notes/slipbox/` or a project in `projects/`.

* If `<name>` matches **only a note**, it targets the note automatically.
* If `<name>` matches **only a project**, it targets the project automatically.
* If `<name>` matches **both a note and a project**, ZettelTeX aborts and requires the `--project` flag to explicitly target the project.

---

## 1. Workspace Initialization & Setup

| Command | Description |
|---|---|
| [`init`](commands/init.md) | Initialize the minimal directory structure and default templates for a new workspace. |
| [`init_config`](commands/init_config.md) | Interactively generate or overwrite `zetteltex.toml`. |

---

## 2. Note Management

| Command | Description |
|---|---|
| [`newnote`](commands/newnote.md) | Create a new atomic note and register it in the database. |
| [`edit`](commands/edit.md) | Open a note in the configured editor. |
| [`rename_note`](commands/rename_note.md) | Interactively rename a note and refactor its labels across the workspace. |
| [`rename_recent`](commands/rename_recent.md) | Rename the *n*-th most recently modified note. |
| [`remove_note`](commands/remove_note.md) | Delete a note and clean up its database records. |
| [`list_recent_files`](commands/list_recent_files.md) | List the most recently modified notes. |
| [`list_unreferenced`](commands/list_unreferenced.md) | Identify notes with no incoming cross-references or inclusions. |
| [`addtodocuments`](commands/addtodocuments.md) | Manually add an `\externaldocument` entry for a note. |
| [`list_citations`](commands/list_citations.md) | List all BibLaTeX citation keys in a note. |

---

## 3. Project Management

| Command | Description |
|---|---|
| [`newproject`](commands/newproject.md) | Create a new project document and register it in the database. |
| [`list_projects`](commands/list_projects.md) | List all projects registered in the database. |
| [`list_project_inclusions`](commands/list_project_inclusions.md) | Show all notes transcluded into a project. |
| [`list_note_projects`](commands/list_note_projects.md) | Show which projects include a given note. |
| [`export_project`](commands/export_project.md) | Bundle a project folder and primary `.tex` file for export. |
| [`export_draft`](commands/export_draft.md) | Expand a draft by inlining transcluded sections. |

---

## 4. Markdown Export

| Command | Description |
|---|---|
| [`export_markdown`](commands/export_markdown.md) | Export a single note or project to Markdown. |
| [`export_all_markdown`](commands/export_all_markdown.md) | Export all notes and projects to Markdown in one pass. |

---

## 5. Rendering & Compilation

| Command | Description |
|---|---|
| [`render`](commands/render.md) | Compile a single note or project to PDF or HTML. |
| [`render_all`](commands/render_all.md) | Compile all notes and projects in parallel. |
| [`render_updates`](commands/render_updates.md) | Incrementally compile only documents that changed. |
| [`biber`](commands/biber.md) | Run the Biber bibliography processor on a document. |

---

## 6. Synchronization & Validation

| Command | Description |
|---|---|
| [`synchronize`](commands/synchronize.md) | Update the database with current note and project metadata. |
| [`force_synchronize`](commands/force_synchronize.md) | Force a full re-index of notes and/or projects (`--notes-only`, `--projects-only`). |
| [`validate_references`](commands/validate_references.md) | Report broken cross-references and missing transclusions. |

---

## 7. Maintenance & Cleanup

| Command | Description |
|---|---|
| [`clean`](commands/clean.md) | Remove orphaned PDF and Markdown files from export directories. |
| [`remove_duplicate_citations`](commands/remove_duplicate_citations.md) | Clean up duplicate citation records in the database. |

---

## 8. Interactive TUI

| Command | Description |
|---|---|
| [`fuzzy`](commands/fuzzy.md) | Launch the interactive fuzzy search interface. |

---

## Related Documentation

* [Configuration Reference](config-reference.md) — Customizing `zetteltex.toml`.
* [Exit Codes](exit-codes.md) — POSIX exit code definitions.
* [Global Options](global-options.md) — Global flags and environment variables.
* [User Guide](../guide/0-getting-started.md) — Step-by-step workflow tutorials.
