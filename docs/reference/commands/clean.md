# `zetteltex clean`
> **Map:** [Command Reference](../commands.md) → **`zetteltex clean`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Removes orphaned PDF and Markdown files from export directories that no longer correspond to any note or project registered in the database.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] clean
```

---

## Options & Flags

This command takes no options or flags.

---

## Behavior & Internal Workflow

1. **Build keep-list**:
   Opens `slipbox.db` and builds a set of valid basenames (`<name>.pdf` and `<name>.md`) for every note and project in the database.
2. **Scan directories**:
   Recursively scans the following directories:
   * Export notes directory (`<obsidian_vault>/<notes_subdir>`)
   * Export projects directory (`<obsidian_vault>/<projects_subdir>`)
   * Legacy `markdown/` directory
   * Legacy `jabberwocky/adjuntos/pdf/` directory
   * Public `pdf/` directory
3. **Remove orphans**:
   Any `.pdf` or `.md` file whose basename is **not** in the keep-list is deleted. Files are kept if they match a tracked note or project, regardless of the directory.
4. **Print summary**:
   Reports the total number of PDFs and Markdown files removed.

> This is useful after renaming or deleting notes and projects, when stale export artifacts would otherwise linger in the vault.

---

## Exit Codes

* **`0`**: Cleanup completed.
* **`1`**: A filesystem or database error occurred.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
zetteltex clean
```

---

## See Also

* [`remove_duplicate_citations`](remove_duplicate_citations.md) — Clean up duplicate citation records.
* [`export_markdown`](export_markdown.md) — Regenerate an individual export.
