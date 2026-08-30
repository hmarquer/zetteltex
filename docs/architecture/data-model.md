# Data Model

> **Map:** [Architecture Overview](overview.md) ← **Data Model** → [Internals / db](../internals/zetteltex-db.md)

ZettelTeX keeps all metadata in a single SQLite file, `slipbox.db` at the workspace root. It is managed entirely by the `zetteltex-db` crate ([Internals / db](../internals/zetteltex-db.md)) — the CLI never writes SQL directly.

## Entities

```
note ────┬── label (labels / targets for links)
         ├── citation (bibliography keys)
         ├── link (source note → target label)
         ├── notetag (join to tag)
project ─┴── inclusion (project → transcluded note)
tag ───────── notetag
```

- **one note can carry many labels, many citations, many outgoing links**
- **one project transcludes many notes** (the same note can appear under several tags/source files)
- **`tag`/`notetag` are defined in the schema but currently unused** by the public API — reserved for future use.

## Tables

| Table | Columns (key ones) | Purpose | Uniqueness |
|---|---|---|---|
| `note` | `id`, `filename`, `title`, `created`, `last_edit_date`, `last_build_date_pdf`, `last_build_date_html` | Atomic notes on disk (`notes/slipbox/`) | `filename` UNIQUE |
| `project` | `id`, `name`, `filename`, `created`, `last_edit_date`, `last_build_date_pdf`, `last_build_date_html` | Project documents | `name`, `filename` UNIQUE |
| `label` | `id`, `note_id`, `label` | Labels a note exposes for cross-referencing | `(note_id, label)` UNIQUE |
| `link` | `id`, `source_id`, `target_id` | Reference edge: source note → target label | `(source_id, target_id)` UNIQUE |
| `citation` | `id`, `note_id`, `citationkey` | Bibliography keys used by a note | `(note_id, citationkey)` UNIQUE |
| `inclusion` | `id`, `project_id`, `note_id`, `source_file`, `tag` | Project transclusions | `(project_id, note_id, source_file, tag)` UNIQUE |
| `tag` | `id`, `name` | Tag vocabulary (unused by API) | `name` UNIQUE |
| `notetag` | `id`, `note_id`, `tag_id` | Note↔tag join (unused by API) | `(note_id, tag_id)` UNIQUE |

## Relationship and timestamps semantics

- `note` and `project` **capture edit times** (`last_edit_date`) and **build times** (`last_build_date_pdf/html`). The staleness check that drives `render_updates` compares `last_edit_date` against the build dates; see [Sync Process](sync-process.md) and [Render Pipeline](render-pipeline.md).
- `last_edit_date` is set from the `.tex` file **mtime during synchronization**, not from file content.
- Every foreign key has **`ON DELETE CASCADE`**, so deleting a note or project automatically removes its labels, citations, links, and inclusions.

## ID policy and upserts

- IDs are SQLite `INTEGER PRIMARY KEY` autoincrement rowids.
- Insert-or-update ("upsert") uses `INSERT ... ON CONFLICT(filename) DO UPDATE` (notes) and `ON CONFLICT(name) DO UPDATE` (projects).
- Replacing child rows (labels, citations, inclusions) is done by delete-then-reinsert, deduplicated, inside an explicit transaction.

## Migrations

The schema is created idempotently with `CREATE TABLE IF NOT EXISTS` on every open (`Database::migrate`). Forward-compatible column additions (`note.title`, `note.last_build_date_html`, `project.last_build_date_html`) are detected via `PRAGMA table_info` and applied with `ALTER TABLE ADD COLUMN`; a lock failure is tolerated and retried on the next open. There are no separate migration files.

> **Note:** The database opens with `journal_mode = DELETE` (WAL is deliberately avoided so no `.db-wal`/`.db-shm` files appear), `synchronous = NORMAL`, `foreign_keys = ON`, and a 5 s busy timeout. Concurrent renders that transiently contend for the DB are retried by the render engine using `BEGIN IMMEDIATE` transactions. See [Internals / db](../internals/zetteltex-db.md).

---

## See Also

- Up: [Architecture Overview](overview.md) — where the database fits
- Down: [Internals / zetteltex-db](../internals/zetteltex-db.md) — schema code, `migrate()`, population query functions
- Lateral: [Sync Process](sync-process.md) — how the tables get populated