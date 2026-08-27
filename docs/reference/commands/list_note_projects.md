# `zetteltex list_note_projects`

Displays all projects that include a specific atomic note via `\transclude`.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] list_note_projects <note>
```

---

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<note>` | string | Yes | Stem filename of the note to inspect. |

---

## Behavior & Internal Workflow

1. Runs workspace synchronization (`synchronize_notes()` and `synchronize_projects()`).
2. Queries the SQLite `inclusion` table joined with `project` to locate all occurrences of `<note>`.
3. Displays the project name, source file, and section tag for each inclusion.

---

## Exit Codes

* **`0`**: Projects listed successfully (or message indicating note is in 0 projects).
* **`1`**: Synchronization or database query error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
zetteltex list_note_projects compactness-in-metric
```

Example output:
```
Projects including note "compactness-in-metric":
1. topology-course/chapter1.tex [tag: definitions]
2. analysis-paper/paper.tex
Total: 2 projects
```

---

## See Also

* [`list_project_inclusions`](list_project_inclusions.md) — List notes included in a project.
* [`list_unreferenced`](list_unreferenced.md) — Check for orphan notes.
