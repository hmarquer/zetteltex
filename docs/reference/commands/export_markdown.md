# `zetteltex export_markdown`
> **Map:** [Command Reference](../commands.md) → **`zetteltex export_markdown`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Exports a single note or project to a Markdown file formatted for Obsidian with YAML frontmatter, backlinks, and PDF preview embeds.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] export_markdown <note> [--project]
```

---

## Arguments & Options

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `<note>` | string | Yes | — | Stem filename of the note or project name to export. |
| `--project` | flag | No | `false` | Force target to be resolved as a project when ambiguous. |

---

## Behavior & Internal Workflow

1. **Target Disambiguation**:
   Resolves whether `<note>` refers to a note in `notes/slipbox/` or a project in `projects/`. If ambiguous, requires `--project`.
2. **Synchronization**:
   Executes `synchronize_notes()` and `synchronize_projects()` to guarantee fresh metadata.
3. **Markdown Generation**:
   * **For Notes**:
     * Generates YAML frontmatter (`title`, `filename`, timestamps, `labels`, `references`, `backlinks`, `citations`, `projects`, `tags`).
     * Injects PDF embeds: `[[<note>.pdf]]` and `![[<note>.pdf]]`.
     * Renders an outgoing references section using relative links, e.g. `` - [target](./target.md) ``.
     * Extracts keyword tags from LaTeX comments (e.g. `% TODO: ...`, `% DEMOSTRACION ...`).
     * Writes to `<obsidian_vault>/<notes_subdir>/<note>.md`.
   * **For Projects**:
     * Generates YAML frontmatter (`title`, `name`, timestamps, `inclusions`, `tags`).
     * Injects PDF embeds: `[[<project>.pdf]]` and `![[<project>.pdf]]`.
     * Generates a grouped list of included notes organized by source subfiles.
     * Writes to `<obsidian_vault>/<projects_subdir>/<project>.md`.

---

## Exit Codes

* **`0`**: Markdown exported successfully.
* **`1`**: Note or project not found, name ambiguity without `--project`, or I/O error.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Export atomic note
zetteltex export_markdown heine-borel-theorem

# Export project
zetteltex export_markdown topology-course --project
```

---

## See Also

* [`export_all_markdown`](export_all_markdown.md) — Export entire workspace to Markdown.
* [Export Guide](../../guide/5-export.md) — Detailed guide on Obsidian integration.
