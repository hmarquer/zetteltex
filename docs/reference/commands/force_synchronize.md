# `zetteltex force_synchronize`

Forces a complete re-parse and database re-index of atomic notes and/or projects in the workspace.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] force_synchronize [OPTIONS]
```

---

## Options & Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--notes-only` | flag | `false` | Force-synchronize only notes in `notes/slipbox/`. |
| `--projects-only` | flag | `false` | Force-synchronize only projects in `projects/`. |

When neither flag is given, both notes and projects are synchronized (notes first, so that projects can resolve their transclusions).

---

## Behavior & Internal Workflow

1. **Notes** (unless `--projects-only`):
   Runs `synchronize_notes` to fully re-index all notes in `notes/slipbox/`, reporting notes synced, links built, and unresolved references.
2. **Projects** (unless `--notes-only`):
   Runs `synchronize_projects` to fully re-index all projects in `projects/` and their inclusions, reporting projects synced, inclusions synced, and inclusions referencing missing notes.

> This is the strongest synchronization command — use it when you suspect the database is stale, after bulk manual edits, or when incremental `synchronize` seems to have missed changes.

---

## Exit Codes

* **`0`**: Requested synchronization completed.
* **`1`**: A parsing or database error occurred.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Force-synchronize notes and projects
zetteltex force_synchronize

# Force-synchronize only notes
zetteltex force_synchronize --notes-only

# Force-synchronize only projects and inclusions
zetteltex force_synchronize --projects-only
```

---

## See Also

* [`synchronize`](synchronize.md) — Incremental synchronization.
* [`validate_references`](validate_references.md) — Report broken cross-references.
