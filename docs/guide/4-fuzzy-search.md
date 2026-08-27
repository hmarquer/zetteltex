# Fuzzy Search: Your Interactive Knowledge Hub

The **Fuzzy Search** interface is the central command hub of ZettelTeX. While traditional note-taking workflows force you to switch context, search file trees, and memorize LaTeX label keys, ZettelTeX provides a zero-latency Terminal User Interface (TUI) designed to be invoked **while writing** without breaking your train of thought.

With instant fuzzy matching, a live LaTeX preview pane, knowledge-graph centrality ranking, and single-keystroke clipboard actions, you can cross-reference, transclude, inspect, or atomize notes in seconds.

---

## The Four Core Workflows

### 1. Rapid Cross-Referencing While Writing
When writing a proof or explanation in your editor and referencing an earlier concept:
1. Open the fuzzy interface (`zetteltex fuzzy` or your global hotkey).
2. Type a partial query (e.g. `heine`).
3. Press **`Ctrl+H`**.
4. The TUI closes instantly. Your system clipboard now contains the complete macro:
   ```latex
   \exhyperref[thm:heine-borel]{heine-borel}{Heine-Borel Theorem}
   ```
5. Paste directly into your document.

If you prefer a plain reference without hyperlinked descriptive text, press **`Ctrl+R`** instead to copy `\excref[thm:heine-borel]{heine-borel}`.

---

### 2. Instant Note Transclusion into Projects
When assembling lecture notes, a chapter, or a paper in a project:
1. Open fuzzy search and find the note you want to include.
2. Read the live preview on the right pane to verify the content.
3. Press **`Ctrl+T`**.
4. The macro `\transclude{heine-borel}` is copied to your clipboard.
5. Paste it into your project `.tex` file.

---

### 3. Extracting Notes from Clipboard (`Ctrl+Alt+N`)
When drafting a long document or lecture summary, you often write definitions or theorems inline that should be turned into standalone atomic notes. ZettelTeX automates this entire extraction:

1. In your editor, copy the LaTeX block containing a `\label{...}`:
   ```latex
   \begin{definition}\label{defn:cauchy-sequence}
   A sequence $(x_n)$ in a metric space $(X, d)$ is called a Cauchy sequence if
   for every $\varepsilon > 0$ there exists $N \in \mathbb{N}$ such that...
   \end{definition}
   ```
2. Open fuzzy search (ensure search bar is empty) and press **`Ctrl+Alt+N`**.
3. ZettelTeX automatically:
   - Extracts the label name (`defn:cauchy-sequence` $\rightarrow$ `cauchy-sequence`).
   - Creates `notes/slipbox/cauchy-sequence.tex` with title "Cauchy sequence" and current date.
   - Injects the definition body into the note template.
   - Registers the note in `slipbox.db` and adds `\externaldocument` to `notes/documents.tex`.
   - Copies `\transclude{cauchy-sequence}` to your clipboard.
   - Opens the new note in your editor.
4. Return to your draft and simply paste `\transclude{cauchy-sequence}` where the definition used to be.

---

### 4. Direct Navigation & Inspection
- **`Ctrl+E`**: Jump straight into your editor (`code`, `nvim`, `vim`) at the selected note or project root.
- **`Ctrl+P`**: Open the compiled PDF in your viewer (`qpdfview`, `zathura`, etc.). If using `qpdfview`, it automatically reuses the existing window in unique mode.
- **`Ctrl+N`**: Create a new empty note named after whatever text is currently typed in the search bar.

---

## Keyboard Shortcuts (TUI)

| Shortcut | Action | Result / Clipboard Content |
|---|---|---|
| **`Ctrl+H`** | Copy `\exhyperref` | `\exhyperref[<best-label>]{<note>}{<Title>}` |
| **`Ctrl+R`** | Copy `\excref` | `\excref[<best-label>]{<note>}` |
| **`Ctrl+T`** | Copy `\transclude` | `\transclude{<note>}` |
| **`Ctrl+E`** | Open in Editor | Opens `.tex` file or project workspace in editor |
| **`Ctrl+P`** | Open PDF | Launches system PDF viewer |
| **`Ctrl+N`** | New Note from Query | Creates note named after search bar text & opens editor |
| **`Ctrl+Alt+N`** | Note from Clipboard | Creates note from clipboard `\label`, injects content, copies `\transclude` |
| **`Up` / `Down`** | Navigate Results | Changes selection; resets preview scroll |
| **`PageUp` / `PageDown`** | Scroll Preview | Scrolls the live preview pane up/down |
| **`Left` / `Right`** | Move Cursor | Navigates the search input bar |
| **`Ctrl+Left` / `Ctrl+Right`** | Jump by Word | Moves cursor word by word in search input |
| **`Home` / `End`** | Jump to Ends | Moves cursor to beginning / end of search bar |
| **`Backspace` / `Delete`** | Edit Search | Deletes characters; updates results in real time |
| **`Esc`** / **`Ctrl+C`** | Quit | Closes interface without taking any action |

---

## How Intelligent Ranking Works

The ranking engine ensures the most relevant notes appear at the top by combining multiple scoring factors into a single unified score:

1. **Name Match** (query vs. note name)
2. **Full-Text Content Match** (query occurrences in `.tex` source)
3. **Graph Popularity** (incoming/outgoing reference counts from `slipbox.db`)
4. **Recency & History** (recently accessed notes, from `.fuzzy_state.json`)

These factors are weighted and merged into a **Unified Score**, which orders the ranked results list.

1. **Name Matching & Fuzzy Distance**:
   - Exact substring match: **+100 pts**.
   - Query contains name: **+80 pts**.
   - Normalized Levenshtein similarity: up to **+50 pts** (tolerates typos and abbreviations).

2. **Full-Text Content Search**:
   - Scans the actual `.tex` source code of all notes in memory.
   - Each occurrence of the query adds **+5 pts** (up to +40 pts).
   - Occurrences in the first 500 characters (e.g. definitions and theorems near the top) receive a **+20 pt bonus**.

3. **Knowledge Graph Centrality (PageRank-style Popularity)**:
   - Notes with dense incoming references (`in_refs`) and outgoing references (`out_refs`) receive up to **+40 pts** of popularity boost. Foundational "hub" notes naturally surface when searching broad terms.

4. **Adaptive History (Empty Search Bar)**:
   - When the search bar is empty, ZettelTeX displays your **recently accessed notes** first, followed by the most popular hubs in your workspace.

---

## Launch Modes & System Integration

### Interactive TUI (Recommended)
```bash
zetteltex fuzzy
```
Opens the full dual-pane interactive interface.

> **Tip: Set up a Global Hotkey**
> For the ultimate zero-friction workflow, bind a system-wide hotkey (e.g. `Super+Z` or `Ctrl+Alt+Z` in your desktop environment / window manager) to spawn a floating terminal running:
> ```bash
> zetteltex --workspace-root /path/to/workspace fuzzy
> ```
> This lets you summon the search, press `Ctrl+H`, and paste the reference into your editor without touching the mouse or leaving your text editor.

### Inline REPL Mode
```bash
zetteltex fuzzy --inline
```
Runs a lightweight, non-fullscreen prompt directly in the current terminal. Ideal for SSH sessions, terminal multiplexers, or quick command-line lookups:
```text
fuzzy> metric compactness
1. compactness-in-metric (145.2)
2. open-covers (112.0)
3. bolzano-weierstrass (94.5)
```


### Headless & Scripted Actions
You can execute any fuzzy action directly from shell scripts, editor plugins (Vim/Neovim Lua, Emacs, VS Code tasks), or keybindings:

```bash
# Copy \exhyperref for best match into clipboard
zetteltex fuzzy --action copy-exhyperref --query "compactness"

# Copy \transclude for a specific note
zetteltex fuzzy --action copy-transclude --item "heine-borel"

# Open a specific note directly in your configured editor
zetteltex fuzzy --action open-editor --item "metric-spaces"

# Open the compiled PDF for a search query
zetteltex fuzzy --action open-pdf --query "riemann-integral"

# Create a note directly from current clipboard content
zetteltex fuzzy --action create-from-clipboard
```

---

## Configuration

Fuzzy search behavior and appearance can be customized in `zetteltex.toml`:

```toml
[fuzzy]
# Maximum search results to display in the list
max_results = 20

# Number of items shown when the search bar is empty (history + popular)
history_results = 20

# TUI accent/selection color (e.g. 'magenta', 'blue', 'green', 'red', 'yellow', 'cyan')
selection_color = "magenta"

# Relative weights for knowledge-graph popularity scoring (optional)
in_refs_weight = 1.0
out_refs_weight = 1.0

# Path to the persistent history and popularity state cache
state_file = ".fuzzy_state.json"
```

---

## Next step

Now that you know how to find, link, and create notes with fuzzy search, learn how to [export notes to Markdown](5-export.md) for visualization in Obsidian.

