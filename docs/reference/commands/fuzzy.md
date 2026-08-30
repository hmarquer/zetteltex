# `zetteltex fuzzy`
> **Map:** [Command Reference](../commands.md) → **`zetteltex fuzzy`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Launches the interactive fuzzy search interface for instantly finding and acting on notes and projects.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] fuzzy [OPTIONS]
```

---

## Options & Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--inline` | flag | `false` | Run a plain-text search prompt in the current terminal instead of the full-screen TUI. |
| `--action <ACTION>` | string | — | *(hidden)* Run a single scripted fuzzy action non-interactively. |
| `--query <TERM>` | string | — | *(hidden)* Search term used with `--action`. |
| `--item <NAME>` | string | — | *(hidden)* Exact item name used with `--action`. |
| `--clipboard-text <TEXT>` | string | — | *(hidden)* Clipboard content override used with `--action`. |

---

## Launch Modes

* **TUI (default)**: When run inside a terminal, opens a full-screen dual-pane interface (search bar + results + preview). When run outside a terminal, attempts to launch a new terminal window running the TUI.
* **Inline (`--inline`)**: Runs a lightweight `fuzzy>` prompt in the current terminal — useful for SSH sessions and multiplexers.
* **Scripted (`--action`)**: Executes one action and returns, for use in keybindings and scripts.

---

## Keyboard Shortcuts (TUI)

| Shortcut | Action |
|---|---|
| `Ctrl+H` | Copy `\exhyperref` to clipboard |
| `Ctrl+R` | Copy `\excref` to clipboard |
| `Ctrl+T` | Copy `\transclude{<note>}` to clipboard |
| `Ctrl+E` | Open selected item in configured editor |
| `Ctrl+P` | Open compiled PDF |
| `Ctrl+N` | Create new note from search bar text |
| `Ctrl+Alt+N` | Create note from clipboard (search bar must be empty) |
| `Up` / `Down` | Navigate results |
| `Left` / `Right` | Move cursor in search bar |
| `Ctrl+Left` / `Ctrl+Right` | Jump by word in search bar |
| `Home` / `End` | Jump to start/end of search bar |
| `PageUp` / `PageDown` | Scroll preview panel |
| `Backspace` / `Delete` | Edit search text |
| `Esc` / `Ctrl+C` | Quit |

---

## Scripted Actions

| Action | Description |
|---|---|
| `copy-exhyperref` | Copy `\exhyperref` for an item (resolved via `--item` or `--query`) |
| `copy-excref` | Copy `\excref` for an item |
| `copy-transclude` | Copy `\transclude{<note>}` for an item |
| `open-editor` | Open an item in the configured editor |
| `open-pdf` | Open the compiled PDF of an item |
| `create-from-query` | Create a new note named after `--query` |
| `create-from-clipboard` | Create a note from the clipboard (expects a `\label{...}` block) |

---

## Exit Codes

* **`0`**: Successful completion (including quitting the TUI without taking an action).
* **`1`**: An error occurred (e.g., empty workspace, missing target for a scripted action, clipboard error).
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Launch the full-screen TUI
zetteltex fuzzy

# Run the inline prompt
zetteltex fuzzy --inline

# Copy an \exhyperref for the best match of a query
zetteltex fuzzy --action copy-exhyperref --query "compactness"

# Open a specific note in the editor
zetteltex fuzzy --action open-editor --item "heine-borel"
```

---

## See Also

* [Fuzzy Search Guide](../../guide/4-fuzzy-search.md) — Detailed workflow and ranking explanation.
* [`edit`](edit.md) — Open a note in the editor.
* [`render`](render.md) — Compile the selected document.
