# `zetteltex list_recent_files`
> **Map:** [Command Reference](../commands.md) → **`zetteltex list_recent_files`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Displays a numbered list of the most recently modified notes in the workspace.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] list_recent_files [n]
```

---

## Arguments

| Argument | Type | Required | Default | Description |
|---|---|---|---|---|
| `[n]` | integer | No | `10` | Maximum number of recent notes to list. |

---

## Output Format

Prints a 1-based indexed table of notes ordered by last modification timestamp:

```
1:	heine-borel-theorem
2:	compactness-in-metric
3:	open-covers
4:	bolzano-weierstrass
5:	metric-spaces
```

The numbers correspond to the positional indices used by [`rename_recent`](rename_recent.md).

---

## Exit Codes

* **`0`**: Notes listed successfully (or empty message printed).
* **`1`**: Filesystem reading error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# List top 10 most recent notes
zetteltex list_recent_files

# List top 5 most recent notes
zetteltex list_recent_files 5
```

---

## See Also

* [`rename_recent`](rename_recent.md) — Rename note by recency index.
* [`edit`](edit.md) — Open most recent note.
