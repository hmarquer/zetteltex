# ZettelTeX VS Code extension

Provides Language Server Protocol support for ZettelTeX, giving contextual
completion while typing in `.tex` notes:

- inside `\excref[<cursor>]{NOTA}` it completes the **labels** of `NOTA`;
- inside `\excref[LABEL]{<cursor>}` it completes **note** names.

The same applies to `\exref` and `\exhyperref`, which share the
`[label]{note}` argument shape.

It launches the `zetteltex lsp` language server (see
`docs/reference/commands/lsp.md` in this repository) over stdio against the
current workspace folder.

It also ships LaTeX snippets for scaffolding link commands:

| Prefix | Command | Placeholders |
|---|---|---|
| `exc` | `\excref` — clever external reference | `{note}` (tab 1), `[label]` (tab 2) |
| `exr` | `\exref` — plain external reference | `{note}` (tab 1), `[label]` (tab 2) |
| `exh` | `\exhyperref` — external hyperreference | `{note}` (tab 1), `[label]` (tab 2), `{text}` (tab 3) |

## Prerequisites

- The `zetteltex` binary must be on your `PATH` (or set
  `zetteltex.lsp.path` to its absolute path in VS Code settings).

## Build & install (development)

```bash
cd editors/vscode
npm install
npm run package
npx @vscode/vsce package   # produces zetteltex-0.1.0.vsix
code --install-extension zetteltex-0.1.0.vsix
```

Reload VS Code, then open a ZettelTeX workspace and start typing
`\excref[` in a note, or use the `exc` / `exr` / `exh` snippets.

## Settings

| Setting | Default | Description |
|---|---|---|
| `zetteltex.lsp.path` | `zetteltex` | Name (on `PATH`) or absolute path to the `zetteltex` executable. |

## Usage

- **Autocomplete**: inside a link-shaped command, the `zetteltex lsp` server
  completes note names in `{...}` and -- once the note is present -- that
  note's labels in `[...]`. See the explanation in the
  [user guide](../../docs/guide/2-linking.md#vs-code-extension).
- **Snippets**: type `exc`, `exr`, or `exh` and accept, then press `Tab` to
  move between the placeholders.
