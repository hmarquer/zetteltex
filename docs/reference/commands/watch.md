# `zetteltex watch`
> **Map:** [Command Reference](../commands.md) → **`zetteltex watch`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Watches for changes to LaTeX files in the workspace and recompiles the affected notes and/or projects.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] watch [NAME] [OPTIONS]
```

* With a `NAME`, watches only that note (or project, with `--project`) and recompiles it on change.
* Without a `NAME`, watches the whole workspace and recompiles everything that went stale.

---

## Options & Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `name` | positional | — | Note or project to watch. Omit to watch the whole workspace. |
| `--project` | flag | `false` | Treat `NAME` as a project. |
| `--format <pdf\|html>` | enum | `pdf` | Output format (`pdf` or `html`). |
| `-j`, `--workers <N>` | integer | `4` | Number of parallel workers (whole-workspace mode). |
| `--poll <MS>` | integer | `800` | Poll interval in milliseconds. |

---

## Behavior & Internal Workflow

1. **Resolve target** (when `NAME` is given):
   Uses `resolve_note_or_project` to decide whether `NAME` is a note or a project. Projects are also detected by their directory under `projects/<name>/`.
2. **Initial render**:
   Compiles the target (or, in whole-workspace mode, everything pending via `render_updates`) once up front, so the latest state is built when the watcher starts.
3. **Poll for changes**:
   Records the modification timestamps of the relevant `.tex` files — for a note, `notes/slipbox/<name>.tex`; for a project, every `.tex` under `projects/<name>/`; for the whole workspace, every `.tex` under `notes/slipbox/` and `projects/`. Every `--poll` milliseconds it re-reads those timestamps and compares them.
4. **Recompile on change**:
   When any watched file's timestamp differs, it recompiles. Targeted mode recompiles just that target; whole-workspace mode re-synchronizes and renders only the stale documents (same staleness logic as `render_updates`).
5. **Keep watching**:
   Recompilation errors are reported but never stop the watcher; it stays alive until interrupted with `Ctrl-C`.

> Output files are never written under `notes/slipbox/` or `projects/`, so recompiling does not re-trigger the watcher.

---

## Exit Codes

* **`0`**: The target did not exist / the watcher was interrupted cleanly.
* **`1`**: A recompilation failed (reported, but watching continues).
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Watch a single note and recompile it (PDF) whenever it changes
zetteltex watch anillo

# Watch a single project as HTML
zetteltex watch --project --format html libro

# Watch the whole workspace and recompile stale documents with 6 workers
zetteltex watch -j 6

# Poll every 300 ms
zetteltex watch --poll 300
```

---

## See Also

* [`render`](render.md) — Compile a document once.
* [`render_updates`](render_updates.md) — Compile only stale documents (reused by whole-workspace watch).
* [`render_all`](render_all.md) — Compile all documents unconditionally.
