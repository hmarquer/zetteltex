# Getting Started
> **Map:** [Guide](0-getting-started.md) → **Getting Started** → [Notes and Projects](1-notes-and-projects.md)

This page covers prerequisites, workspace creation, and initial configuration. Once done, proceed to [Notes and Projects](1-notes-and-projects.md) to create your first documents.

## Prerequisites

| Dependency | Required for | Check |
|---|---|---|
| [Rust](https://rustup.rs/) (stable) | Building from source | `rustc --version` |
| `pdflatex` | PDF rendering | `pdflatex --version` |
| `biber` | Bibliography (optional) | `biber --version` |
| `make4ht` | HTML rendering (optional) | `make4ht --version` |

If you install from a prebuilt binary, Rust is not required at runtime.

## Create a workspace

A **workspace** is a directory that contains your notes, projects, templates, and a local SQLite database. ZettelTeX needs this structure to locate your files and track metadata.

```bash
mkdir my-zettelkasten && cd my-zettelkasten
zetteltex init
```

This creates:

```
my-zettelkasten/
├── notes/
│   ├── slipbox/          # Individual notes (.tex)
│   └── documents.tex     # Master document for cross-references
├── projects/             # Multi-note documents (.tex per project)
└── template/             # LaTeX templates (copied from built-in defaults)
    ├── note.tex
    ├── project.tex
    ├── style.sty
    ├── ztxbase.sty
    ├── texbook.cls
    └── texnote.cls
```

The templates are embedded in the binary and copied non-destructively — if you edit them later, `init` will not overwrite your changes.

## Generate configuration

```bash
zetteltex init_config
```

This creates `zetteltex.toml` at the workspace root. The interactive prompt asks for your preferred editor, interface language, output directories, and fuzzy search settings. Press Enter to accept any default. If `zetteltex.toml` already exists, you will be asked whether to overwrite it.

The full list of settings:

| Setting | Default | Purpose |
|---|---|---|
| `lang` | `en` | Interface language (`es` or `en`) |
| `editor` | `code` | Preferred editor (`code`, `vim`, `nvim`, or custom path) |
| `pdf_output_dir` | `pdf` | Directory for rendered PDFs |
| `html_output_dir` | `html` | Directory for rendered HTML |
| `obsidian_vault` | `vault` | Obsidian vault name for export |
| `notes_subdir` | `notes` | Subdirectory for exported note Markdown |
| `projects_subdir` | `projects` | Subdirectory for exported project Markdown |
| `max_results` | `20` | Maximum fuzzy search results |
| `history_results` | `20` | Number of history entries in fuzzy |

> **Tip:** If you plan to use Markdown export with Obsidian, set `pdf_output_dir` to a path **inside** `obsidian_vault`. This allows Obsidian to embed PDFs correctly via relative paths. For example, if your vault is at `~/Documents/my-vault`, set `pdf_output_dir` to `~/Documents/my-vault/latex/pdf`.

You can also create the file manually — if it is missing or malformed, ZettelTeX falls back to built-in defaults.

## Shell tab completion (zsh)

If you use zsh (e.g. via [oh-my-zsh](https://ohmyz.sh/)), you can enable Tab completion for the `zetteltex` CLI: subcommand names are completed automatically, and for commands that take a note or project name (`render`, `edit`, `biber`, `export_markdown`, `list_citations`, `rename_note`, `list_project_inclusions`, …) the candidates are read from `notes/slipbox/*.tex` and `projects/*` in the current workspace.

The completion script lives at `completions/_zetteltex` in this repository. Install it:

```bash
mkdir -p "${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/completions"
cp completions/_zetteltex "${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/completions/"
autoload -U compinit && compinit
```

Restart your shell and try it: inside a workspace, type `zetteltex render an<Tab>` to see matching notes and projects. With `--project` on the line, only projects are offered.

## Next step

Once your workspace is ready, proceed to [Notes and Projects](1-notes-and-projects.md) to learn about the two document types and how to create them.

## See Also

* [Command Reference](../reference/commands.md) — every command and flag.
* [Architecture Overview](../architecture/overview.md) — how ZettelTeX is structured.
