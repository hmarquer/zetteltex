# Exit Codes

ZettelTeX follows POSIX exit code conventions to distinguish between successful operations, general runtime errors, and workspace structural failures.

---

## Summary Table

| Exit Code | Name | Meaning | Common Causes |
|---|---|---|---|
| `0` | **Success** | The command completed successfully | Normal termination, `--help`, cancelled interactive prompts |
| `1` | **Execution Error** | A runtime, compilation, or logic error occurred | Missing note/project, duplicate name, broken references, LaTeX/Biber compilation failure |
| `2` | **Workspace Error** | Workspace discovery or structure validation failed | Invalid `--workspace-root`, missing `notes/slipbox/`, `projects/`, or `template/` |

---

## Detailed Exit Codes

### Exit Code 0 — Success

Indicates that the command completed all tasks without errors.

#### Scenarios returning 0:
* A note or project was successfully created, rendered, exported, or deleted.
* `zetteltex --help` or `zetteltex --version` was executed.
* `zetteltex validate_references` found zero broken references.
* `zetteltex init_config` was cancelled by the user (`n` at the overwrite confirmation).
* `zetteltex` was invoked without arguments (prints command hint).

---

### Exit Code 1 — Execution Error

Indicates that the requested command encountered an operational error, failed an assertion, or encountered external tool errors.

#### Scenarios returning 1:
* **Duplicate entity**: Attempting to create a note or project whose name already exists in the SQLite database or filesystem.
* **Non-existent entity**: Referencing a note or project that cannot be found during `edit`, `remove_note`, `rename_note`, or `list_citations`.
* **LaTeX / Biber build failure**: An underlying `pdflatex`, `biber`, or `make4ht` process exited with a non-zero code or could not be found in `PATH`.
* **Validation failures**: `zetteltex validate_references` detected one or more broken references (`missing_note`, `missing_label`).
* **Interactive index out of bounds**: Passing an invalid numerical index to `rename_recent`.

#### Example error output:
```bash
$ zetteltex newnote existing-note
Error: A note with file name existing-note already exists in the database. If this is not the case then run zetteltex synchronize and try again
$ echo $?
1
```

---

### Exit Code 2 — Workspace Discovery Error

Indicates that ZettelTeX failed to locate or validate a valid workspace root.

Before running any subcommand (except `init`), ZettelTeX executes `WorkspacePaths::discover()`, which verifies the presence of:
1. `notes/slipbox/`
2. `projects/`
3. `template/`

If any of these required directories are missing from the target directory, execution halts immediately with exit code `2`.

#### Example error output:
```bash
$ cd /tmp && zetteltex list_projects
Error de workspace: working directory not found: /tmp/notes/slipbox, /tmp/projects, /tmp/template. Check --workspace-root and the minimal structure (notes/slipbox, projects, template)
$ echo $?
2
```

#### Resolution:
* Verify that you are running ZettelTeX from inside your workspace root.
* Pass the explicit root path via `--workspace-root <PATH>`.
* Initialize the missing structure with `zetteltex init`.

---

## Integration and Scripting Examples

### Bash Error Handling

```bash
#!/usr/bin/env bash
set -e

# Run validation and abort if broken references are detected
zetteltex validate_references

# Render only updated documents
zetteltex render_updates
```

### Differentiating Error Types in CI/CD

```bash
#!/usr/bin/env bash

zetteltex validate_references
STATUS=$?

case $STATUS in
  0)
    echo "✓ Workspace references are valid."
    ;;
  1)
    echo "✗ Broken references or compilation issues found!"
    exit 1
    ;;
  2)
    echo "✗ Fatal: Invalid ZettelTeX workspace directory!"
    exit 2
    ;;
  *)
    echo "✗ Unknown error ($STATUS)"
    exit $STATUS
    ;;
esac
```

### Git Pre-Commit Hook

You can configure a Git pre-commit hook (`.git/hooks/pre-commit`) to prevent committing broken links:

```bash
#!/usr/bin/env bash
echo "Running ZettelTeX reference validation..."
zetteltex validate_references --notes-only

if [ $? -ne 0 ]; then
    echo "Commit aborted: Please fix broken references before committing."
    exit 1
fi
```

---

## Related Documentation

* [Command Reference](commands.md) — Comprehensive reference for all subcommands.
* [Global Options](global-options.md) — Global flags including `--workspace-root`.
* [Troubleshooting Guide](../guide/7-troubleshooting.md) — Detailed steps to diagnose and recover from errors.
