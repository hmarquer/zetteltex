# `zetteltex addtodocuments`

Manually adds an `\externaldocument` declaration for a note to the master `notes/documents.tex` index.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] addtodocuments <name>
```

---

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<name>` | string | Yes | Stem filename of the note to register in `notes/documents.tex`. |

---

## Behavior & Internal Workflow

ZettelTeX manages cross-document LaTeX linking using the `xr` / `zref-xr` package mechanism. For note $A$ to resolve labels from note $B$, note $B$'s `.aux` file must be registered in `notes/documents.tex`.

While [`newnote`](newnote.md) adds this entry automatically, `addtodocuments` allows you to register manually created `.tex` files or restore missing entries:

1. Formats the macro: `\externaldocument[<name>-]{<name>}`.
2. Reads `notes/documents.tex`.
3. If the line is not already present, appends it and saves the file.

---

## Exit Codes

* **`0`**: Entry registered successfully (or already present).
* **`1`**: Filesystem I/O error reading/writing `notes/documents.tex`.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Manually register a note created outside ZettelTeX
zetteltex addtodocuments custom-external-note
```

---

## See Also

* [`newnote`](newnote.md) — Create and auto-register a note.
* [`validate_references`](validate_references.md) — Check cross-reference validity.
