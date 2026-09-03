# `zetteltex lsp`
> **Map:** [Command Reference](../commands.md) → **`zetteltex lsp`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Runs a **Language Server Protocol (LSP)** server over stdin/stdout. It is meant to be launched by an editor integration (currently a minimal VS Code extension) to provide **contextual completion** while you type in a `.tex` note.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] lsp
```

---

## What it completes

Inside a link-shaped LaTeX command — `\excref[LABEL]{NOTA}`, `\exref`, `\exhyperref` — the server answers `textDocument/completion` depending on where the cursor is:

| Cursor location | Completion offered |
|---|---|
| Inside the `[...]` slot (`\excref[<cursor>]{NOTA}`) | The **labels** defined in `NOTA` (from `\label` / `\currentdoc`) |
| Inside the `{...}` slot (`\excref[LABEL]{<cursor>}`) | The **note names** available in `notes/slipbox` |

Completions are filtered by the text already typed (case-insensitive) and the inputs are 1-indexed line / UTF-16 character positions.

> **Experimental.** Currently only `textDocument/completion` (plus the standard lifecycle: `initialize`, `initialized`, `shutdown`, `exit`, `textDocument/didOpen|didChange|didClose`, which feed the server the live document text) is implemented.

---

## Workspace resolution

The server keeps the `--workspace-root` it was launched with. If none is given it falls back to the first **workspace folder** announced by the client during `initialize`. It reads notes/labels directly from `notes/slipbox/*.tex` on demand (no database access needed).

---

## Editor integration

See [`editors/vscode/`](../../../editors/vscode/README.md) for the extension:

```bash
cd editors/vscode
npm install && npm run compile
npx @vscode/vsce package
code --install-extension zetteltex-0.1.0.vsix
```

The extension launches `zetteltex lsp --workspace-root <folder>` (PATH, or configured via `zetteltex.lsp.path`).

---

## Exit Codes

* **`0`**: Server shut down cleanly (client sent `shutdown` + `exit`).
* **`1`**: Protocol/handshake error.

---

## See Also

* [`edit`](edit.md) — open a note/project in the external editor.
* [Configuration Reference](../config-reference.md) — the `[general] editor` value used elsewhere.
