# `zetteltex export_project`

Exports a project into a self-contained standalone directory by resolving and inlining all `\transclude` note contents.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] export_project <folder> [texfile]
```

---

## Arguments

| Argument | Type | Required | Default | Description |
|---|---|---|---|---|
| `<folder>` | string | Yes | — | Project folder name inside `projects/`. |
| `[texfile]` | string | No | `<folder>.tex` | Name of the primary project `.tex` file. |

---

## Behavior & Internal Workflow

1. Locates the source project file at `projects/<folder>/<texfile>`.
2. Creates an output directory at `projects/<folder>/standalone/`.
3. Reads the project source line-by-line:
   * Detects `\transclude[tag]{note}` or `\transclude{note}` statements.
   * Reads the target note file from `notes/slipbox/<note>.tex`.
   * If a tag is specified (e.g. `[definitions]`), extracts the `%<*definitions>...%</definitions>` block. If omitted, extracts the `%<*note>...%</note>` block.
   * Inlines the extracted LaTeX content directly into the destination file.
4. Writes the expanded standalone file to `projects/<folder>/standalone/<texfile>`.

---

## Exit Codes

* **`0`**: Standalone project exported successfully.
* **`1`**: Source file not found, transcluded note missing, or tagged block missing.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Export project 'topology-course' to standalone/
zetteltex export_project topology-course

# Export with custom main tex file
zetteltex export_project topology-course master.tex
```

---

## See Also

* [`export_draft`](export_draft.md) — Export arbitrary draft file with inlined notes.
* [`export_markdown`](export_markdown.md) — Export project to Obsidian Markdown.
