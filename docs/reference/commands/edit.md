# `zetteltex edit`
> **Map:** [Command Reference](../commands.md) → **`zetteltex edit`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Opens a note or project in your configured text editor. If no name is specified and
`--project` is not used, it automatically opens the most recently modified note.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] edit [name] [--project]
```

---

## Arguments

| Argument | Type | Required | Default | Description |
|---|---|---|---|---|
| `[name]` | string | No | Most recent note | Stem filename of the note (or project with `--project`) to edit (without `.tex`). |
| `--project` | flag | No | `false` | Treat `name` as a project. When set, `[name]` becomes required. |

---

## Editor Resolution Strategy

ZettelTeX uses the `[general] editor` value in `zetteltex.toml` (e.g., `"code"`, `"vim"`, `"nvim"`, `"hx"`). If no editor is configured, the command fails with a message directing you to run `zetteltex init_config`.

The configured command is spawned with the target file path passed as its final argument.

---

## Behavior & Internal Workflow

1. If `[name]` is omitted (without `--project`):
   * Scans `notes/slipbox/` for `.tex` files.
   * Sorts files by filesystem modification time (`mtime`) descending.
   * Selects the most recent file.
2. Resolves whether `name` refers to a note or a project:
   * A note resolves to `notes/slipbox/<name>.tex`.
   * A project resolves to `projects/<name>/<name>.tex`.
   * If a name matches both a note and a project, an error asks you to use `--project`.
3. Spawns the editor process attached to the terminal.

---

## Exit Codes

* **`0`**: File opened in editor successfully.
* **`1`**: Name not found, `--project` used without a name, or editor process failed to start.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Open specific note
zetteltex edit topology-basics

# Open a project
zetteltex edit my-project --project

# Open the note modified most recently
zetteltex edit
```

---

## See Also

* [`newnote`](newnote.md) — Create a new note.
* [`newproject`](newproject.md) — Create a new project.
* [`list_recent_files`](list_recent_files.md) — View recent notes.
* [Configuration Reference](../config-reference.md) — Configure default editor.