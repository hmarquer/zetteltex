# `zetteltex newproject`
> **Map:** [Command Reference](../commands.md) → **`zetteltex newproject`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Creates a new multi-note project in `projects/` and registers it in the SQLite database.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] newproject <name>
```

---

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<name>` | string | Yes | Project name (used for folder name and main `.tex` document). |

---

## Behavior & Internal Workflow

1. **Uniqueness Check**:
   Queries `slipbox.db` (`db.project_id_by_name(project_name)`). If a project with this name already exists, the command aborts with an error.
2. **Directory & File Creation**:
   * Creates `projects/<name>/`.
   * Reads `template/project.tex`.
   * Derives project title from `<name>`, leaving `\date{\today}` (the compile-time date) untouched.
   * Writes the resulting LaTeX file to `projects/<name>/<name>.tex`.
3. **Database Registration**:
   Inserts the project into `slipbox.db` (`db.upsert_project()`) with filename `<name>.tex` and current timestamp.

---

## Exit Codes

* **`0`**: Project created successfully.
* **`1`**: Project name already exists, template missing, or I/O error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Create a new project for a topology course
zetteltex newproject topology-course

# Creates:
# projects/topology-course/topology-course.tex
```

---

## See Also

* [`list_projects`](list_projects.md) — List all projects.
* [`list_project_inclusions`](list_project_inclusions.md) — View notes transcluded in a project.
* [`newnote`](newnote.md) — Create an atomic note.
