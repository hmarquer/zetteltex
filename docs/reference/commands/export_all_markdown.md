# `zetteltex export_all_markdown`
> **Map:** [Command Reference](../commands.md) → **`zetteltex export_all_markdown`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Exports all notes and projects in the workspace to Obsidian-compatible Markdown files in a single pass.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] export_all_markdown [OPTIONS] [--no-keywords] [--output-dir <DIR>]
```

---

## Options & Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `-n`, `--notes` | boolean flag | `false` | Export only atomic notes. |
| `-p`, `--projects` | boolean flag | `false` | Export only projects. |
| `--no-keywords` | boolean flag | `false` | Omit the generated `## Etiquetas` keyword section in every exported file. The frontmatter `tags:` lists are unaffected. |
| `--output-dir <DIR>` | path | `[export] obsidian_vault` | Directory replacing the configured vault. It can be absolute or relative to the workspace root; `notes_subdir` / `projects_subdir` are still appended below it. |

If neither `--notes` nor `--projects` is passed, both notes and projects are exported.

---

## Behavior & Internal Workflow

1. Runs `synchronize_notes()` and `synchronize_projects()`.
2. Creates destination directories, where `<output-root>` is `--output-dir` when provided, otherwise `obsidian_vault`:
   * `<output-root>/<notes_subdir>/`
   * `<output-root>/<projects_subdir>/`
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

# Export everything without keyword sections into a temporary vault layout
zetteltex export_all_markdown --no-keywords --output-dir /tmp/review-vault
```

---

## See Also

* [`export_markdown`](export_markdown.md) — Export a single note or project.
* [`clean`](clean.md) — Clean orphaned Markdown exports.
