# Linking Notes
> **Map:** [Guide](0-getting-started.md) → **Linking Notes** → [Rendering](3-rendering.md)

The zettelkasten method is built on linking notes. ZettelTeX provides custom LaTeX commands that handle cross-references between notes automatically. These commands work seamlessly whether the target is in the same note or in a different one.

## Cross-referencing notes

### `\excref` — Cross-reference a note

```latex
See \excref{topology-basics} for the definition.
```

This renders as a hyperlinked reference to the note `topology-basics`. No label is needed inside the target note — ZettelTeX resolves it automatically via the `\externaldocument` mechanism.

If you want to reference a specific labeled element inside a note (e.g. a definition or theorem), use the optional argument:

```latex
See \excref[defn:compact-space]{topology-basics} for the definition.
```

### `\exref` — Plain cross-reference

```latex
As shown in \exref{topology-basics}.
```

Same as `\excref` but uses a plain `\ref` instead of `\cref` (no automatic type prefix). Supports the same optional argument for specific labels.

### `\exhyperref` — Custom display text

```latex
See \exhyperref{topology-basics}{the compactness note} for details.
```

Renders as a hyperlink with custom display text. The optional argument works the same way:

```latex
See \exhyperref[defn:compact-space]{topology-basics}{the compactness definition} for details.
```

## Embedding note content

### `\transclude` — Embed a note inside another document

```latex
\transclude{topology-basics}
```

Embeds the full content of another note inline, as if you had pasted it. This command is primarily intended for **projects** — it lets you compose a project document from atomic notes without duplicating content.

You can also include only a tagged section:

```latex
% In topology-basics.tex, surround content with tags:
%<*definitions>
\label{defn:compact-space}
A topological space is compact if...
%</definitions>

% In the project:
\transclude[definitions]{topology-basics}
```

## Synchronize

After editing notes and adding links, synchronize the metadata to the database:

```bash
zetteltex synchronize
```

This parses every `.tex` file in `notes/slipbox/` and `projects/`, extracts `\label`, `\ref`, `\cite`, and `\transclude` commands, and updates the SQLite database. Run `synchronize` after any significant edit so that cross-references and `\transclude` resolve correctly.

## Validate references

To catch broken cross-references:

```bash
zetteltex validate_references
```

Reports any `\excref`, `\exref`, or `\transclude` that points to a missing note or label. Run this after `synchronize` to verify that all links are valid before rendering.

## VS Code extension

An official VS Code extension (see [`editors/vscode/`](../../editors/vscode/README.md)) makes linking notes much faster. It provides **contextual autocomplete** and **link snippets** while you type in `.tex` files.

### Install

1. Build and package the extension from source (the `zetteltex` binary must already be on your `PATH`, or you can point the extension at an absolute path — see [Settings](#settings)):

   ```bash
   cd editors/vscode
   npm install
   npm run package
   npx @vscode/vsce package   # produces zetteltex-0.1.0.vsix
   ```

2. Install the packaged extension:

   ```bash
   code --install-extension zetteltex-0.1.0.vsix
   ```

3. Reload VS Code (`Developer: Reload Window`), open a ZettelTeX workspace, and edit a note in `notes/slipbox/`. The `zetteltex` server starts automatically the first time you edit a `.tex` file.

### Use — contextual autocomplete

While the cursor is inside a link-shaped command, the extension offers context-aware completions backed by the `zetteltex lsp` server:

- `\excref[<typing>]{NOTA}` completes the **labels** of `NOTA`;
- `\excref[LABEL]{<typing>}` completes the **note names**.

The same applies to `\exref` and `\exhyperref`, which share the `[label]{note}` argument shape.

The typical flow to insert a labelled reference is:

1. Type the opening brace of the note: `\excref{` — the note-name dropdown appears.
2. Pick the note (e.g. `topology-basics`) — the server fills it in, inserting a closing `}` for you.
3. Type `[` after the command to open the label slot — the dropdown now proposes that note's labels.
4. Pick a label — it is inserted in the `[...]` slot.

A "(sin etiqueta)" / "(no label)" entry is always offered, so you can reference the note itself without a specific label.

> Because the label slot `[...]` precedes the note `{...}` in the syntax, the
> label dropdown only has items once the note name is already present.

### Use — snippets

The extension also ships three LaTeX snippets that scaffold a whole reference command. Type the prefix and press <kbd>Tab</kbd> (or <kbd>Enter</kbd>), then use <kbd>Tab</kbd> to jump between the placeholders:

| Prefix | Command | Placeholders |
|---|---|---|
| `exc` | `\excref` — clever external reference | `{note}` (tab 1), `[label]` (tab 2) |
| `exr` | `\exref` — plain external reference | `{note}` (tab 1), `[label]` (tab 2) |
| `exh` | `\exhyperref` — external hyperreference with custom text | `{note}` (tab 1), `[label]` (tab 2), `{display text}` (tab 3) |

For example, typing `exc` and accepting produces:

```latex
\excref[note]{}
```

with the cursor on the `{note}` placeholder (`$1`); the label slot already shows the sample text `note` (`$2`). Press <kbd>Tab</kbd> to move to the `[label]` slot and type over it. After filling in all placeholders, <kbd>Tab</kbd> once more places the cursor after the trailing space so you can keep typing.

### Settings

| Setting | Default | Description |
|---|---|---|
| `zetteltex.lsp.path` | `zetteltex` | Name (on `PATH`) or absolute path to the `zetteltex` executable. |

Snippets and autocomplete both target the `latex` language, so they work in any `.tex` or `.sty` document, inside or outside a ZettelTeX workspace. The LSP completion, however, needs a valid ZettelTeX workspace (its `notes/slipbox/` and `projects/`) to resolve notes and labels.

## Next step

After synchronizing, learn how to [render](3-rendering.md) your documents to PDF or HTML.

## See Also

* [Reference / `synchronize`](../reference/commands/synchronize.md) — keeping the database in sync.
* [Sync Process](../architecture/sync-process.md) — how links are discovered and stored.
* [VS Code extension](../../editors/vscode/README.md) — autocomplete and snippets for `\excref`/`\exref`/`\exhyperref`.
