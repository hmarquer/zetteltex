# `zetteltex export_draft`
> **Map:** [Command Reference](../commands.md) → **`zetteltex export_draft`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Expands and inlines all `\transclude` macros from an arbitrary input LaTeX file into an output draft file.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] export_draft <input_file> <output_file>
```

---

## Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `<input_file>` | string | Yes | Path to the source LaTeX file to expand. |
| `<output_file>` | string | Yes | Destination path for the expanded draft. |

---

## Behavior & Internal Workflow

1. Reads `<input_file>`.
2. Replaces every occurrence of `\transclude[tag]{note}` or `\transclude{note}` with the corresponding block extracted from `notes/slipbox/<note>.tex`.
3. Writes the fully expanded document to `<output_file>`.

---

## Exit Codes

* **`0`**: Draft exported successfully.
* **`1`**: Source file missing, transcluded note missing, or tagged block missing.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Expand a project document into a single standalone draft
zetteltex export_draft projects/topology-course/topology-course.tex draft.tex
```

---

## See Also

* [`export_project`](export_project.md) — Export a project directory to `standalone/`.
* [`export_markdown`](export_markdown.md) — Export to Markdown.
