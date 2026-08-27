# `zetteltex export_all_markdown`

Exports all notes and projects in the workspace to Obsidian-compatible Markdown files in a single pass.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] export_all_markdown [OPTIONS]
```

---

## Options & Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--notes` | boolean flag | `false` | Export only atomic notes. |
| `--projects` | boolean flag | `false` | Export only projects. |

If neither `--notes` nor `--projects` is passed, both notes and projects are exported.

---

## Behavior & Internal Workflow

1. Runs `synchronize_notes()` and `synchronize_projects()`.
2. Creates destination directories:
   * `<obsidian_vault>/<notes_subdir>/`
   * `<obsidian_vault>/<projects_subdir>/`
3. Iterates over all notes and projects in `slipbox.db`, generating complete Markdown files with YAML frontmatter, backlinks, and embeds.

---

## Exit Codes

* **`0`**: All matching items exported successfully.
* **`1`**: Synchronization or file generation error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Export all notes and projects
zetteltex export_all_markdown

# Export only notes
zetteltex export_all_markdown --notes

# Export only projects
zetteltex export_all_markdown --projects
```

---

## See Also

* [`export_markdown`](export_markdown.md) — Export a single note or project.
* [`clean`](clean.md) — Clean orphaned Markdown exports.
