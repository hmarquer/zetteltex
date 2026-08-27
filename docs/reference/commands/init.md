# `zetteltex init`

Initializes the minimal workspace directory structure and default LaTeX templates for a new ZettelTeX knowledge base.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] init
```

---

## Arguments & Options

This command takes no arguments or specific options. It respects the global [`--workspace-root`](../global-options.md) option.

---

## Behavior & Internal Workflow

When `init` is invoked, ZettelTeX executes `init_workspace()`:

1. **Creates Workspace Directories**:
   * `notes/slipbox/` — Stores all atomic `.tex` note files.
   * `projects/` — Stores multi-note project folders and master documents.
   * `template/` — Stores workspace LaTeX templates and document classes.

2. **Populates LaTeX Templates**:
   Copies built-in template files embedded into the binary into `template/` (only if they do not already exist):
   * `note.tex` — Template used when creating new atomic notes with `newnote`.
   * `project.tex` — Template used when creating new projects with `newproject`.
   * `style.sty` — Shared styling, packages, and custom macros (`\excref`, `\exref`, `\exhyperref`, `\transclude`).
   * `texbook.cls` — Document class for projects.
   * `texnote.cls` — Document class for atomic notes.

3. **Initializes Master Document**:
   Creates `notes/documents.tex` with initial comments if missing. This file acts as the master LaTeX registry for `\externaldocument` cross-references.

4. **Non-Destructive Guarantee**:
   If directories or template files already exist, `init` preserves their contents without overwriting.

---

## Exit Codes

* **`0`**: Workspace initialized successfully.
* **`1`**: Filesystem I/O error during directory or template creation.

---

## Examples

```bash
# Initialize a new workspace in the current directory
mkdir my-zettelkasten && cd my-zettelkasten
zetteltex init

# Initialize a workspace at a specific destination path
zetteltex --workspace-root ~/documents/notes init
```

---

## See Also

* [`init_config`](init_config.md) — Interactive configuration wizard.
* [Getting Started Guide](../../guide/0-getting-started.md) — Workspace initialization guide.
