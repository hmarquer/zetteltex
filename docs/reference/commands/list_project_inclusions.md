# `zetteltex list_project_inclusions`
> **Map:** [Command Reference](../commands.md) → **`zetteltex list_project_inclusions`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Displays all atomic notes transcluded (`\transclude{...}`) into a specific project.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] list_project_inclusions <project>
```

---

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<project>` | string | Yes | Name of the project to inspect. |

---

## Behavior & Internal Workflow

1. Runs `synchronize_notes()` and `synchronize_projects()` to guarantee up-to-date inclusion tables.
2. Queries SQLite table `inclusion` for entries associated with `<project>`.
3. Displays:
   * Included note filename
   * Section tag (if transcluded via `\transclude[tag]{note}`)
   * Project source file where the `\transclude` macro appears
   * Total number of included notes

---

## Exit Codes

* **`0`**: Inclusions listed successfully.
* **`1`**: Synchronization or database query error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
zetteltex list_project_inclusions topology-course
```

Example output:
```
Inclusions in project "topology-course":
1. metric-spaces (in) topology-course.tex
2. compactness-in-metric [tag: definitions] (in) chapter1.tex
3. heine-borel-theorem (in) chapter1.tex
Total: 3 notes included
```

---

## See Also

* [`list_note_projects`](list_note_projects.md) — Find which projects include a note.
* [`export_draft`](export_draft.md) — Export a project with inlined transclusions.
