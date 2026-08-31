# `zetteltex synchronize`
> **Map:** [Command Reference](../commands.md) → **`zetteltex synchronize`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Parses all notes and projects on disk and updates the SQLite database with their labels, cross-references, citations, and transclusions.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] synchronize
```

---

## Options & Flags

This command takes no options or flags.

---

## Behavior & Internal Workflow

1. **Synchronize notes**:
   Scans every `.tex` file in `notes/slipbox/`, parses `\label{...}`, `\ref{...}`, `\cite{...}`, `\excref`, `\exref`, `\exhyperref` commands, and updates the database. Records the number of notes synced, links built, and unresolved references.
2. **Synchronize projects**:
   Scans every project in `projects/`, parses `\transclude{...}` inclusions, and updates the database. Records the number of projects synced, inclusions synced, and inclusions referencing missing notes.
3. **Print summary**:
   Reports the aggregated counts for both notes and projects.

> Run `synchronize` after editing files externally, or before `render_updates`, `export_all_markdown`, or `validate_references`, so the database reflects the current state on disk.

---

## Exit Codes

* **`0`**: Synchronization completed successfully.
* **`1`**: A parsing or database error occurred.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
zetteltex synchronize
```

---

## See Also

* [`force_synchronize`](force_synchronize.md) — Force a full re-scan of notes and/or projects (`--notes`, `--projects`).
* [`validate_references`](validate_references.md) — Report broken cross-references.
