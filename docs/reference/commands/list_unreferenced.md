# `zetteltex list_unreferenced`
> **Map:** [Command Reference](../commands.md) → **`zetteltex list_unreferenced`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Identifies and displays orphan notes that have no incoming cross-references and are not included in any project.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] list_unreferenced
```

---

## Arguments & Options

This command takes no arguments or specific options. It respects the global [`--workspace-root`](../global-options.md) option.

---

## Behavior & Internal Workflow

1. Runs an automated `synchronize_notes()` pass to guarantee up-to-date link and inclusion tables.
2. Queries SQLite for all notes meeting both criteria:
   * **Zero incoming links**: No other note contains an `\excref`, `\exref`, or `\exhyperref` pointing to this note.
   * **Zero project inclusions**: No project includes this note via `\transclude`.
3. Displays a numbered list of unreferenced note filenames.

---

## Exit Codes

* **`0`**: Command completed successfully.
* **`1`**: Synchronization or database query error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
zetteltex list_unreferenced
```

Example output:
```
1: draft-ideas
2: temporary-lemma
```

---

## See Also

* [`validate_references`](validate_references.md) — Check for broken outgoing references.
* [`remove_note`](remove_note.md) — Delete unneeded orphan notes.
