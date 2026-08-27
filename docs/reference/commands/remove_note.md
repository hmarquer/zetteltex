# `zetteltex remove_note`

Deletes an atomic note from the filesystem, removes its indexing entry from `notes/documents.tex`, and cleans up all database records.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] remove_note <name>
```

---

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<name>` | string | Yes | Stem filename of the note to delete. |

---

## Behavior & Internal Workflow

When `remove_note` is invoked, ZettelTeX performs the following actions:

1. **Deletes File**:
   Removes `notes/slipbox/<name>.tex` from disk if it exists.
2. **Cleans Master Index**:
   Removes the matching `\externaldocument[<name>-]{<name>}` line from `notes/documents.tex`.
3. **Database Deletion with Cascades**:
   Executes `db.delete_note(name)` on `slipbox.db`.
   Because foreign keys are configured with `ON DELETE CASCADE`:
   * All associated `label` rows for this note are deleted.
   * All outgoing `link` rows from this note are deleted.
   * All `citation` rows for this note are deleted.
   * All `inclusion` rows referencing this note in projects are deleted.
   * All `notetag` rows linking this note are deleted.

> **Warning:** Deleting a note does not remove references to it in other `.tex` files. Run [`zetteltex validate_references`](validate_references.md) after deleting a note to detect broken cross-references.

---

## Exit Codes

* **`0`**: Note removed successfully.
* **`1`**: Database error or filesystem deletion error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Delete the note 'scratch-idea'
zetteltex remove_note scratch-idea
```

---

## See Also

* [`clean`](clean.md) — Remove orphaned PDF and Markdown exports.
* [`validate_references`](validate_references.md) — Detect dangling links after note deletion.
* [`list_unreferenced`](list_unreferenced.md) — Identify orphan notes.
