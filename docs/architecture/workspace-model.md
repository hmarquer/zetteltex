# Workspace Model

> **Map:** [Architecture Overview](overview.md) ← **Workspace Model** → [Internals / core](../internals/zetteltex-core.md)

A **workspace** is a directory on disk with a fixed, minimal structure. Everything ZettelTeX does — synchronizing, rendering, exporting — resolves against this layout.

## On-disk layout

```
my-workspace/
├── notes/
│   ├── slipbox/          # Individual atomic notes (.tex)
│   └── documents.tex     # Master document for cross-references
├── projects/             # Multi-note documents, one folder each
│   └── <project>/        #   main file: <project>/<project>.tex
├── template/             # LaTeX templates (note.tex, project.tex, style.sty,
│                         #   ztxbase.sty, texbook.cls, texnote.cls)
├── slipbox.db            # SQLite metadata database (created/opened by zetteltex-db)
└── zetteltex.toml        # Optional configuration (created by `init_config`)
```

## Discovery and validation (`zetteltex-core`)

The `WorkspacePaths` struct ([`crates/zetteltex-core/src/lib.rs`](../internals/zetteltex-core.md)) computes the three key directories from a root and validates them:

- `notes/slipbox`
- `projects`
- `template`

`WorkspacePaths::discover` fails with a clear error if any of the three directories is missing. This is why every command (except `init`) requires a valid workspace. The default root is the current directory; `--workspace-root` overrides it.

Paths are stored as `PathBuf` and joined lazily, so the code never hand-concatenates path strings. See [Internals / zetteltex-core](../internals/zetteltex-core.md) for the exact struct and the `validate_component_name` path-traversal guard.

## Note vs. project

Two document types share the workspace:

- **Note** → `notes/slipbox/<name>.tex`
- **Project** → `projects/<name>/<name>.tex`

The disambiguation rule (which name wins when both exist) is documented once in the [Command Reference name-disambiguation note](../reference/commands.md#name-disambiguation-rule) and implemented in `resolve_note_or_project` in the CLI ([Internals / cli](../internals/zetteltex-cli.md)).

## Where templates come from

`template/` is created by `zetteltex init` from templates embedded in the binary (`include_str!`). `init` copies them non-destructively — editing them later survives a re-run of `init`.

---

## See Also

- Up: [Architecture Overview](overview.md) — crate responsibilities
- Down: [Internals / zetteltex-core](../internals/zetteltex-core.md) — `WorkspacePaths`, discovery, validation
- Lateral: [Reference / render command](../reference/commands/render.md) — note vs project resolution from the user side