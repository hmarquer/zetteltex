# Sync Process

> **Map:** [Architecture Overview](overview.md) ← **Sync Process** → [Data Model](data-model.md) → [Internals / cli sync](../internals/zetteltex-cli.md)

The **sync** pipeline reconciles what is on disk (`notes/slipbox/*.tex`, `projects/*/*.tex`) with what is in `slipbox.db`. It is the primary writer of the database and a prerequisite for rendering, exporting, and validation.

## Command surface

- `zetteltex synchronize` → runs both phases below and prints a summary.
- `zetteltex force_synchronize` → same, with `--notes-only` / `--projects-only`.
- `render_updates`, `validate_references`, and `export_*` **implicitly synchronize first**, so users get fresh metadata without a manual step.

## Phase 1 — notes (`synchronize_notes`)

1. Open `slipbox.db` and start a **transaction** (`TransactionGuard`, a RAII guard that commits explicitly and rolls back on drop).
2. Purge leftover temporary render notes (prefix `.zetteltex-render-`).
3. Walk `notes/slipbox/`:
   - read each `.tex`, run `parse_note` (zetteltex-parser) to extract `labels`, `citations`, `references` (`\excref`/`\exhyperref`/`\exref`), `plain_refs` (`\ref`);
   - derive `last_edit_date` from the file **mtime**;
   - `upsert_note` + `replace_labels` + `replace_citations`.
4. `clear_links()`, then a **second pass** resolves every outgoing reference into a `link` row (source note → target label). References to labels the target doesn't have are counted as unresolved (not an error at this stage).
5. Commit; return a `SyncStats` summary.

## Phase 2 — projects (`synchronize_projects`)

1. Open `slipbox.db` + transaction.
2. For each `projects/<name>/` using `<name>/<name>.tex` as the main file: `upsert_project`.
3. Recursively collect the project's `.tex` files (`collect_tex_files`, skipping symlinks) and run `parse_project_inclusions` per file to extract `\transclude[tag]{note}` pairs.
4. `replace_project_inclusions`: a transclusion to a note that does not exist is a **fatal** error for `synchronize` (`resolve_note_id` requires an exact note-name match).

## What sync does NOT do

- It does **not** render anything.
- It does **not** validate semantics: references to missing notes/labels are surfaced only by `validate_references` (see below), except for project transclusions, which sync rejects up front.

## Validation

`zetteltex validate_references` re-reads the synced database and reports broken references: a `\excref`/`\exhyperref` to a nonexistent note (`missing_note`) or label (`missing_label`), an internal `\ref` to a label missing from the same file (or, inside a project, from any file of that project), and a `\transclude` to a nonexistent note (`missing_note`). Checks are filtered by `--notes-only` / `--projects-only`.

## Failure model

- A missing note referenced by `\transclude` in a project → `synchronize` itself errors with details (the strictest failure).
- Other unresolved references are counted and reported; `validate_references` is the dedicated saw for those cases.

---

## See Also

- Up: [Architecture Overview](overview.md) — pipeline list
- Down: [Data Model](data-model.md) — the tables sync fills (`label`, `link`, `citation`, `inclusion`)
- Lateral: [Internals / CLI](../internals/zetteltex-cli.md) — `synchronize_notes`/`synchronize_projects`/validation on disk