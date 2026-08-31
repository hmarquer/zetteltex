# Internals — `zetteltex-db`

> **Map:** [Architecture Overview](../architecture/overview.md) → [Internals](functions.md) → **zetteltex-db** → [Generated rustdoc](https://docs.rs/zetteltex_db)

`zetteltex-db` owns the entire SQLite persistence layer: schema creation, migrations, and ~50 query/mutation methods on `Database`. The CLI never writes SQL directly — all database access goes through this crate, in a single `src/lib.rs` (914 lines).

## Public entry points

| Item                | Signature                                               | Purpose                                                                                                                        |
| ------------------- | ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `init_database`     | `pub fn init_database(path: &Path) -> Result<Database>` | Convenience wrapper over `Database::open`; what the CLI calls at workspace root `slipbox.db`                                   |
| `Database::open`    | `pub fn open(path: &Path) -> Result<Self>`              | Open connection, set pragmas (busy timeout 5 s, `foreign_keys=ON`, `synchronous=NORMAL`, `journal_mode=DELETE`), run `migrate` |
| `Database::migrate` | `pub fn migrate(&self) -> Result<()>`                   | Idempotent `CREATE TABLE IF NOT EXISTS` for all 10 tables + forward-compatible column adds                                     |
| `DbError`           | `pub enum DbError`                                      | Structured error (thiserror) with `Db`/`Parse`/`Other` variants; the only doc-commented item in the crate                      |

## Query/mutation families

The ~50 `Database` methods by group:

- **Upserts**: `upsert_note` (ON CONFLICT filename), `upsert_project` (ON CONFLICT name).
- **Lookups**: `note_id_by_filename`, `note_title_by_filename`, `note_metadata_by_filename`, `note_exists`, `note_last_edit_date`, `project_id_by_name`, `project_metadata_by_name`, `list_notes`, `list_projects`, `labels_for_note`, `citations_for_note`, `note_keywords`, `project_keywords`, `target_label_id`, `label_exists`, `list_note_projects`, `list_project_inclusions_by_name`, `note_popularity_stats`.
- **Link graph**: `notes_referencing_note`, `list_unreferenced_notes`, `clear_links`, `insert_link` (INSERT OR IGNORE).
- **Replace-after-delete (idempotent)**: `replace_labels`, `replace_citations`, `replace_note_keywords`, `replace_project_keywords`, `replace_project_inclusions`, `remove_duplicate_citations`.
- **Build state**: `notes_needing_render`, `notes_needing_render_html`, `projects_needing_render`, `projects_needing_render_html`, `note_has_citations`, `set_note_last_build_date_pdf/html`, `set_project_last_build_date_pdf/html`.
- **Transactions**: `begin_transaction` (`BEGIN IMMEDIATE`), `commit_transaction`, `rollback_transaction`.
- **Delete/rename**: `delete_note_by_filename`, `delete_notes_with_prefix`, `rename_note_filename`.

## The load-bearing staleness SQL

The whole incremental-render feature rests on one private function, `needing_render_generic` (`lib.rs:634`):

```sql
SELECT {key} FROM {table}
WHERE {build} IS NULL
   OR last_edit_date IS NULL
   OR last_edit_date > {build}
ORDER BY {key} ASC
```

An item needs rendering when it was **never built** (`last_build_date_pdf/html IS NULL`) **or** its edit date is unknown **or** its last edit is **newer** than its last build. Edit dates are captured from `.tex` mtimes during sync; build dates are stamped after a successful compile. It is a database-timestamp comparison — the filesystem mtime is only used separately, for backlink pre-rendering in the CLI. See [Render Pipeline](../architecture/render-pipeline.md) and [Sync Process](../architecture/sync-process.md).

## Concurrency story

- `busy_timeout = 5 s` at open; parallel renders can contend on `slipbox.db`, handled by the CLI's `run_with_sqlite_lock_retry` (see [Internals / cli](zetteltex-cli.md)).
- **WAL is deliberately avoided** (`journal_mode = DELETE`) so no `.db-wal`/`.db-shm` files appear next to the database; if the journal-mode switch fails because the file is locked mid-run, the open continues (retried next time).
- `BEGIN IMMEDIATE` transactions (via `TransactionGuard`) keep sync writes from deadlocking against concurrent readers.

## Schema and migrations

The 10 tables (`note`, `project`, `label`, `link`, `citation`, `inclusion`, `tag`, `notetag`, `note_keyword`, `project_keyword`) are created idempotently on every open. Forward-compatible column additions (`note.title`, `note.last_build_date_html`, `project.last_build_date_html`) are detected with `PRAGMA table_info` and applied with `ALTER TABLE ADD COLUMN`; lock failures are ignored and retried. `tag`/`notetag` exist in the schema but have no public API usage — reserved for future work. `note_keyword`/`project_keyword` store the detected keywords (`keyword` + trailing `value`) per note/project, populated during sync. Full detail in [Data Model](../architecture/data-model.md).

## File map

| Task | Start reading at |
|---|---|
| Opening/creating the database | `crates/zetteltex-db/src/lib.rs` — `open`, pragmas |
| Schema and migrations | `crates/zetteltex-db/src/lib.rs` — `migrate` |
| Staleness queries | `crates/zetteltex-db/src/lib.rs` — `needing_render_generic`, `notes_needing_render*` |
| Adding a column compatibly | `crates/zetteltex-db/src/lib.rs` — `column_exists` + the `ALTER TABLE` block in `migrate` |

---

## See Also

- Up: [Data Model](../architecture/data-model.md) — the schema this crate owns
- Down: [Generated rustdoc](https://docs.rs/zetteltex_db) — signatures
- Lateral: [Sync Process](../architecture/sync-process.md) — what populates these tables; [Render Pipeline](../architecture/render-pipeline.md) — what reads build timestamps