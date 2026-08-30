# `zetteltex remove_duplicate_citations`
> **Map:** [Command Reference](../commands.md) → **`zetteltex remove_duplicate_citations`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Removes duplicate citation records from the database that may have been accumulated during note processing.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] remove_duplicate_citations
```

---

## Options & Flags

This command takes no options or flags.

---

## Behavior & Internal Workflow

1. Opens `slipbox.db`.
2. Calls `db.remove_duplicate_citations()` to delete redundant citation rows.
3. Reports the number of duplicates removed, or "No duplicate citations found" if there was nothing to clean.

---

## Exit Codes

* **`0`**: Maintenance completed.
* **`1`**: A database error occurred.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
zetteltex remove_duplicate_citations
```

---

## See Also

* [`clean`](clean.md) — Remove orphaned PDF/Markdown export files.
* [`list_citations`](list_citations.md) — Inspect citations in a specific note.
