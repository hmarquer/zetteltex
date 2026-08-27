# `zetteltex init_config`

Interactively generates or updates the `zetteltex.toml` configuration file in the workspace root.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] init_config
```

---

## Arguments & Options

This command takes no arguments or specific options. It respects the global [`--workspace-root`](../global-options.md) option.

---

## Interactive Prompts

When invoked, `init_config` prompts the user step-by-step in the terminal. Pressing `Enter` accepts the bracketed default value for each prompt:

1. **Overwrite Check** *(only if `zetteltex.toml` exists)*:
   `The file .../zetteltex.toml already exists. Do you want to overwrite it? (y/N)`
   Entering `n` cancels the operation safely without modifying the existing file.
2. **Interface language** (`lang`): `es` (Spanish) or `en` (English) [default: `en`].
3. **Preferred editor** (`editor`): Editor executable or path (`code`, `vim`, `nvim`, `hx`) [default: `code`].
4. **PDF output directory** (`pdf_output_dir`): Directory for compiled PDF files [default: `pdf`].
5. **HTML output directory** (`html_output_dir`): Directory for compiled HTML documents [default: `html`].
6. **Obsidian vault path** (`obsidian_vault`): Destination Obsidian vault root [default: `vault`].
7. **Notes export subdirectory** (`notes_subdir`): Subdirectory inside the vault for notes [default: `notes`].
8. **Projects export subdirectory** (`projects_subdir`): Subdirectory inside the vault for projects [default: `projects`].
9. **Fuzzy maximum results** (`max_results`): Number of matches shown in TUI search [default: `20`].
10. **Fuzzy history results** (`history_results`): Number of notes shown on empty query [default: `20`].
11. **Selection accent color** (`selection_color`): TUI theme color (`magenta`, `blue`, `green`, `red`, etc.) [default: `magenta`].

---

## Exit Codes

* **`0`**: Configuration generated successfully, or operation cancelled safely by user (`n` to overwrite).
* **`1`**: Filesystem I/O error writing `zetteltex.toml`.
* **`2`**: Workspace discovery error (workspace root missing required directories).

---

## Examples

```bash
# Run interactive configuration wizard
zetteltex init_config
```

---

## See Also

* [Configuration Reference](../config-reference.md) — Comprehensive explanation of all `zetteltex.toml` fields.
* [`init`](init.md) — Initialize workspace directory structure.
