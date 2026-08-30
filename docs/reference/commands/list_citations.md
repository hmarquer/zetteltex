# `zetteltex list_citations`
> **Map:** [Command Reference](../commands.md) → **`zetteltex list_citations`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Parses a note and lists all unique BibLaTeX citation keys (`\cite{...}`) referenced inside it.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] list_citations <name>
```

---

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<name>` | string | Yes | Stem filename of the note to inspect. |

---

## Behavior & Internal Workflow

1. Verifies that the note exists in `slipbox.db`.
2. Reads `notes/slipbox/<name>.tex`.
3. Invokes `zetteltex_parser::parse_note()`, which strips LaTeX comments and matches citation macros (`\cite{...}`, `\parencite{...}`, `\textcite{...}`, etc.).
4. Collects and deduplicates all comma-separated keys.
5. Prints each unique citation key on a new line.

---

## Exit Codes

* **`0`**: Citations parsed and printed successfully.
* **`1`**: Note not found in database or file reading error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
zetteltex list_citations heine-borel-theorem
```

Example output:
```
rudin1976principles
munkres2000topology
```

---

## See Also

* [`biber`](biber.md) — Compile bibliography for a note.
* [`remove_duplicate_citations`](remove_duplicate_citations.md) — Clean duplicate database citation records.
