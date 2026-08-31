# `zetteltex newnote`
> **Map:** [Command Reference](../commands.md) → **`zetteltex newnote`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Creates a new atomic LaTeX note in `notes/slipbox/`, registers it in the SQLite database, and indexes it in `notes/documents.tex`.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] newnote <name>
```

---

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<name>` | string | Yes | The stem filename of the note (without the `.tex` extension). |

---

## Behavior & Internal Workflow

When `newnote` is executed, ZettelTeX performs the following operations:

1. **Uniqueness Check**:
   Queries `slipbox.db` (`db.note_exists(note_name)`). If a note with this filename already exists, the command aborts with an error message.
2. **Template Expansion**:
   * Reads `template/note.tex`.
   * Formats the document title from `<name>` (replaces hyphens/underscores with spaces and capitalizes the words).
   * Injects the current date in `dd/mm/yyyy` format into `\date{}`.
   * Writes the resulting LaTeX file to `notes/slipbox/<name>.tex`. If the file already exists on disk, template copying is skipped.
3. **Master Document Registration**:
   Appends `\externaldocument[<name>-]{<name>}` to `notes/documents.tex` if not already present. This enables cross-referencing to labels inside this note from any other document.
4. **Database Registration**:
   Executes `db.upsert_note()` in `slipbox.db`, recording the filename, derived title, and creation timestamp.

---

## Exit Codes

* **`0`**: Note created and registered successfully.
* **`1`**: Note already exists, missing template file, or filesystem I/O error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Create an atomic note named 'cauchy-schwarz-inequality'
zetteltex newnote cauchy-schwarz-inequality

# Creates: notes/slipbox/cauchy-schwarz-inequality.tex
# Appends to notes/documents.tex: \externaldocument[cauchy-schwarz-inequality-]{cauchy-schwarz-inequality}
```

---

## See Also

* [`edit`](edit.md) — Open note in configured editor.
* [`rename_note`](rename_note.md) — Rename note and refactor labels.
* [`remove_note`](remove_note.md) — Delete note and cleanup database.
* [`newproject`](newproject.md) — Create a multi-note project.
