# `zetteltex export_markdown`
> **Map:** [Command Reference](../commands.md) → **`zetteltex export_markdown`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Exports a single note or project to a Markdown file formatted for Obsidian with YAML frontmatter, backlinks, and PDF preview embeds.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] export_markdown <note> [--project] [--no-keywords] [--output-dir <DIR>]
```

---

## Arguments & Options

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `<note>` | string | Yes | — | Stem filename of the note or project name to export. |
| `--project` | flag | No | `false` | Force target to be resolved as a project when ambiguous. |
| `--no-keywords` | flag | No | `false` | Omit the generated `## Etiquetas` keyword section. The frontmatter `tags:` list is unaffected. |
| `--output-dir <DIR>` | path | No | `[export] obsidian_vault` | Directory replacing the configured vault. It can be absolute or relative to the workspace root; `notes_subdir` / `projects_subdir` are still appended below it. |

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
      * Reads keyword tags from the database (populated during `synchronize_notes`/`synchronize_projects`; keywords are configured via `[keywords] list` in `zetteltex.toml`). Use `--no-keywords` to omit the `## Etiquetas` section.
      * Writes to `<output-root>/<notes_subdir>/<note>.md`, where `<output-root>` is `--output-dir` when provided, otherwise `obsidian_vault`.

   * **For Projects**:
     * Generates YAML frontmatter (`title`, `name`, timestamps, `inclusions`, `tags`).
     * Injects PDF embeds: `[[<project>.pdf]]` and `![[<project>.pdf]]`.
     * Generates a grouped list of included notes organized by source subfiles.
      * Reads keyword tags from the database (scanned across every `.tex` file in the project folder during sync). Use `--no-keywords` to omit the `## Etiquetas` section.
      * Writes to `<output-root>/<projects_subdir>/<project>.md`, where `<output-root>` is `--output-dir` when provided, otherwise `obsidian_vault`.


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

# Export without the keyword section, into a temporary vault layout
zetteltex export_markdown heine-borel-theorem --no-keywords --output-dir /tmp/review-vault
```

---

## See Also

* [`export_all_markdown`](export_all_markdown.md) — Export entire workspace to Markdown.
* [Export Guide](../../guide/5-export.md) — Detailed guide on Obsidian integration.
