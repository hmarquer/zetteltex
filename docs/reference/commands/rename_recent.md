# `zetteltex rename_recent`

Renames the $n$-th most recently modified note without having to specify its current name.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] rename_recent [n]
```

---

## Arguments

| Argument | Type | Required | Default | Description |
|---|---|---|---|---|
| `[n]` | integer | No | `1` | 1-based index of the recent note to rename ($n \ge 1$). |

---

## Behavior & Internal Workflow

1. Runs an internal `synchronize_notes()` pass to ensure SQLite timestamps and state are current.
2. Scans `notes/slipbox/` and sorts notes by last filesystem modification date descending.
3. Retrieves the $n$-th note.
4. Prompts the user: `Change file name to [<current_name>]: `.
5. If a new name is entered:
   * Renames the `.tex` file in `notes/slipbox/`.
   * Updates `notes/documents.tex`.
   * Updates the `note` table in `slipbox.db`.
   * Refactors all incoming `\excref`, `\exref`, `\exhyperref`, and `\transclude` macros across all notes and projects in the workspace.

---

## Exit Codes

* **`0`**: Note renamed successfully, or no change made.
* **`1`**: Index $n = 0$, index out of bounds, note not found, or filesystem error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Rename the note modified most recently
zetteltex rename_recent

# Rename the 3rd most recent note
zetteltex rename_recent 3
```

---

## See Also

* [`rename_note`](rename_note.md) — Rename a note by explicit name and refactor labels.
* [`list_recent_files`](list_recent_files.md) — View index numbers of recent notes.
