# `zetteltex edit`

Opens a note in your configured text editor. If no note name is specified, it automatically opens the most recently modified note.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] edit [name]
```

---

## Arguments

| Argument | Type | Required | Default | Description |
|---|---|---|---|---|
| `[name]` | string | No | Most recent note | Stem filename of the note to edit (without `.tex`). |

---

## Editor Resolution Strategy

ZettelTeX uses the `[general] editor` value in `zetteltex.toml` (e.g., `"code"`, `"vim"`, `"nvim"`, `"hx"`). If no editor is configured, the command fails with a message directing you to run `zetteltex init_config`.

The configured command is spawned with the target `.tex` path passed as its final argument.

---

## Behavior & Internal Workflow

1. If `[name]` is omitted:
   * Scans `notes/slipbox/` for `.tex` files.
   * Sorts files by filesystem modification time (`mtime`) descending.
   * Selects the most recent file.
2. Resolves target path: `notes/slipbox/<name>.tex`.
3. Verifies file existence. If missing, exits with an error.
4. Spawns the editor process attached to the terminal.

---

## Exit Codes

* **`0`**: Note opened in editor successfully.
* **`1`**: Note file not found or editor process failed to start.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Open specific note
zetteltex edit topology-basics

# Open the note modified most recently
zetteltex edit
```

---

## See Also

* [`newnote`](newnote.md) — Create a new note.
* [`list_recent_files`](list_recent_files.md) — View recent notes.
* [Configuration Reference](../config-reference.md) — Configure default editor.
