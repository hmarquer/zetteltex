# `zetteltex list_keywords`
> **Map:** [Command Reference](../commands.md) → **`zetteltex list_keywords`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Lists notes and/or projects that carry a given keyword, together with the comment text that follows the keyword on the line. With no keyword it lists **all** detected keywords.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] list_keywords [<keyword>] [--notes] [--projects]
```

---

## Arguments & Options

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `<keyword>` | string | No | all | Keyword to filter by (name, with or without trailing `:`). If omitted, lists every detected keyword. |
| `--notes` | flag | No | `false` | List only notes. |
| `--projects` | flag | No | `false` | List only projects. |

If neither `--notes` nor `--projects` is given, both notes and projects are listed. If only `--notes` is given, notes are listed; if only `--projects`, projects are listed.

---

## Behavior & Internal Workflow

1. Runs `synchronize_notes()` and `synchronize_projects()` so the keyword data in `slipbox.db` is up to date.
2. Queries `slipbox.db` with `db.list_note_keywords(<keyword>?)` and `db.list_project_keywords(<keyword>?)` (optional filter, `keyword = None` means all).
3. Prints the matching notes (prefixed `-`) and projects (prefixed `*`) as `name  #KEYWORD comment` lines, grouped into "Notas con keyword" / "Proyectos con keyword" sections.
4. Prints a message when nothing matches.

Keywords and their comments are detected during sync from the `[keywords] list` in `zetteltex.toml` and stored in the `note_keyword` / `project_keyword` tables. See [Configuration Reference](../config-reference.md).

---

## Exit Codes

* **`0`**: Query completed (with or without matches).
* **`1`**: Database/re-sync error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Every keyword in notes and projects
zetteltex list_keywords

# Every TODO across the whole workspace
zetteltex list_keywords TODO

# TODOs in projects only
zetteltex list_keywords TODO --projects

# Any keyword in notes only
zetteltex list_keywords --notes
```

Example output (`zetteltex list_keywords TODO`):
```
Notas con keyword ("TODO"):
- compactness  #TODO pollish proof
Proyectos con keyword ("TODO"):
* topology-course  #TODO add diagrams
```

---

## See Also

* [`export_markdown`](export_markdown.md) — Export notes/projects to Markdown, which renders keywords as `#KEYWORD` tags.
* [Configuration Reference](../config-reference.md) — `[keywords] list` configuration.
