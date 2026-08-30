# Configuration Reference (`zetteltex.toml`)
> **Map:** [Reference](README.md) → **Configuration Reference** → [Commands](commands.md)

ZettelTeX is configured via a TOML file named `zetteltex.toml` located at the root of the workspace.

If `zetteltex.toml` is missing or any individual field is omitted, ZettelTeX automatically falls back to built-in defaults. You can generate a starter configuration file interactively by running `zetteltex init_config`.

---

## File Structure Overview

```toml
[general]
lang = "en"
editor = "code"

[render]
pdf_output_dir = "pdf"
html_output_dir = "html"

[export]
obsidian_vault = "vault"
notes_subdir = "notes"
projects_subdir = "projects"

[fuzzy]
max_results = 20
history_results = 20
in_refs_weight = 1.5
out_refs_weight = 1.0
selection_color = "magenta"
```

---

## Configuration Sections and Fields

### `[general]`

General application preferences.

| Field | Type | Default | Description |
|---|---|---|---|
| `lang` | string | `"en"` | Interface and messaging language. Supported values: `"en"` (English) or `"es"` (Spanish). If omitted, English is used. |
| `editor` | string | `""` | Command or executable path used by `zetteltex edit` (e.g. `"code"`, `"vim"`, `"nvim"`, `"hx"`). Required for `edit` to work; if omitted, `zetteltex edit` fails with a message pointing to `zetteltex init_config`. |
| `author` | string | `""` | Default author name written into the `\author{}` command of every new note (`newnote`) and project (`newproject`). Empty (or omitted) keeps the author declared in the templates. |

#### Example:
```toml
[general]
lang = "en"
editor = "nvim"
author = "Ada Lovelace"
```

---

### `[render]`

Compilation and output directories for rendered documents.

| Field | Type | Default | Description |
|---|---|---|---|---|
| `pdf_output_dir` | string | `"pdf"` | Directory where compiled PDF files are saved. Relative paths are resolved against the workspace root. |
| `html_output_dir` | string | `"html"` | Directory where compiled HTML documents and assets are saved. Relative paths are resolved against the workspace root. |
| `allow_shell_escape` | boolean | `false` | Pass `-shell-escape`/`--shell-escape` to `pdflatex`/`make4ht`. **Security risk:** allows `.tex` notes to run arbitrary OS commands via `\write18` (e.g. `\immediate\write18{...}`), so a note you did not author could execute code with your privileges. Leave off unless a document genuinely needs it. |
| `render_timeout_secs` | integer | `120` | Time limit (in seconds) applied to each invocation of an external tool (`pdflatex`, `make4ht`, `biber`). If a tool does not finish within this time it is killed and the render fails, preventing a hung compilation from blocking the CLI forever. Set to a larger value if your projects legitimately take longer to compile; omit or set to `null` to use the default. |

#### Example:
```toml
[render]
pdf_output_dir = "build/pdf"
html_output_dir = "build/html"
allow_shell_escape = false
render_timeout_secs = 300
```

> **Tip for Obsidian integration:** If you want Obsidian to display compiled PDF previews inline, set `pdf_output_dir` to a path inside your Obsidian vault (e.g., `vault/pdf`).

---

### `[export]`

Settings for exporting LaTeX documents to Markdown (e.g. for Obsidian vaults).

| Field | Type | Default | Description |
|---|---|---|---|
| `obsidian_vault` | string | `"jabberwocky"` | Root directory of your Obsidian vault. Can be an absolute path or relative to the workspace root. |
| `notes_subdir` | string | `"latex/zettelkasten"` | Subdirectory inside `obsidian_vault` where exported note Markdown files (`.md`) are placed. |
| `projects_subdir` | string | `"latex/asignaturas"` | Subdirectory inside `obsidian_vault` where exported project Markdown files (`.md`) are placed. |

#### Example:
```toml
[export]
obsidian_vault = "~/Documents/ObsidianVault"
notes_subdir = "zettelkasten"
projects_subdir = "projects"
```

---

### `[fuzzy]`

Settings for the Ratatui-based interactive fuzzy search interface (`zetteltex fuzzy`).

| Field | Type | Default | Description |
|---|---|---|---|
| `max_results` | integer | `50` (`20` in `init_config`) | Maximum number of search results displayed in the results list. |
| `history_results` | integer | `10` (`20` in `init_config`) | Number of recent and popular notes displayed when the search query is empty. |
| `in_refs_weight` | float | `1.5` | Multiplier for incoming cross-references when computing note graph centrality. |
| `out_refs_weight` | float | `1.0` | Multiplier for outgoing cross-references when computing note graph centrality. |
| `selection_color` | string | `"magenta"` | Accent color for UI borders and highlighted selections. Supported values: `"magenta"`, `"blue"`, `"green"`, `"red"`, `"cyan"`, `"yellow"`, `"white"`. |
| `state_file` | string | `""` | Optional path to a custom state file for caching search history and popularity scores. |

#### Popularity Score Formula:
When the fuzzy search query is empty or matches multiple items equally, notes are ranked by network centrality:
$$\text{Score} = (\text{Incoming References} \times \text{in\_refs\_weight}) + (\text{Outgoing References} \times \text{out\_refs\_weight})$$

#### Example:
```toml
[fuzzy]
max_results = 25
history_results = 15
in_refs_weight = 2.0
out_refs_weight = 1.0
selection_color = "cyan"
```

---

## Interactive Generation

To create or regenerate `zetteltex.toml` using guided prompts:

```bash
zetteltex init_config
```

The interactive wizard asks for values with sensible defaults. If `zetteltex.toml` already exists, you will be prompted before it is overwritten.

---

## See Also

* [Getting Started Guide](../guide/0-getting-started.md) — Initial workspace setup.
* [Command Reference](commands.md) — All subcommands and CLI options.
* [Fuzzy Search Guide](../guide/4-fuzzy-search.md) — Detailed explanation of TUI keybindings and ranking.
