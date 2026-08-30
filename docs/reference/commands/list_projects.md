# `zetteltex list_projects`
> **Map:** [Command Reference](../commands.md) → **`zetteltex list_projects`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Displays all projects registered in the workspace SQLite database.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] list_projects
```

---

## Arguments & Options

This command takes no arguments or specific options. It respects the global [`--workspace-root`](../global-options.md) option.

---

## Behavior & Internal Workflow

Queries `slipbox.db` (`db.list_projects()`) and prints a 1-indexed list of registered project names.

---

## Exit Codes

* **`0`**: Projects listed successfully.
* **`1`**: Database query error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
zetteltex list_projects
```

Example output:
```
Projects:
1:	topology-course
2:	analysis-paper
3:	differential-geometry
```

---

## See Also

* [`newproject`](newproject.md) — Create a new project.
* [`list_project_inclusions`](list_project_inclusions.md) — View notes included in a project.
