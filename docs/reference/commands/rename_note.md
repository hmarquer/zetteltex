# `zetteltex rename_note`
> **Map:** [Command Reference](../commands.md) → **`zetteltex rename_note`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Interactively renames a note file and refactors its labels and cross-references across the entire workspace.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] rename_note <name>
```

---

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<name>` | string | Yes | Stem filename of the existing note to rename. |

---

## Behavior & Internal Workflow

`rename_note` performs a two-stage interactive refactoring process:

### 1. Note Filename Refactoring
* Prompts the user: `Rename file '<name>' [leave empty to skip]: `.
* If a new filename `<new_name>` is entered:
  * Renames `notes/slipbox/<name>.tex` $\to$ `notes/slipbox/<new_name>.tex`.
  * Updates `notes/documents.tex`: replaces `\externaldocument[<name>-]{<name>}` with `\externaldocument[<new_name>-]{<new_name>}`.
  * Updates `slipbox.db`: updates table `note` and cascades updates to foreign keys.
  * **Global Macro Refactoring**: Scans all `.tex` files in `notes/slipbox/` and `projects/`, rewriting:
    * `\excref{<name>}` $\to$ `\excref{<new_name>}`
    * `\excref[label]{<name>}` $\to$ `\excref[label]{<new_name>}`
    * `\exref[label]{<name>}` $\to$ `\exref[label]{<new_name>}`
    * `\exhyperref[label]{<name>}{text}` $\to$ `\exhyperref[label]{<new_name>}{text}`
    * `\transclude[tag]{<name>}` $\to$ `\transclude[tag]{<new_name>}`

### 2. Label Refactoring
* Queries `slipbox.db` for all `\label{...}` entries defined in the note.
* For each label, prompts: `Rename label '<label>' in '<effective_name>' [leave empty to skip]: `.
* If a new label name is entered:
  * Updates the definition inside the note `.tex` file.
  * Updates `slipbox.db` (`db.rename_label()`).
  * Scans all workspace `.tex` files and updates all `\excref[old_label]{...}`, `\exref[old_label]{...}`, and `\exhyperref[old_label]{...}` invocations to use `new_label`.

---

## Exit Codes

* **`0`**: Note and/or labels renamed successfully, or operation skipped.
* **`1`**: Note not found in database, target filename collision, or I/O error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Rename note and refactor its labels
zetteltex rename_note cauchy-schwarz

# Prompts:
# Rename file 'cauchy-schwarz' [leave empty to skip]: cauchy-schwarz-inequality
# Rename label 'thm:cs' in 'cauchy-schwarz-inequality' [leave empty to skip]: thm:cauchy-schwarz
```

---

## See Also

* [`rename_recent`](rename_recent.md) — Rename note by recency index.
* [`remove_note`](remove_note.md) — Delete note and remove records.
* [`validate_references`](validate_references.md) — Validate cross-reference integrity.
