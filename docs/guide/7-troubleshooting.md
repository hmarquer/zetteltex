# Troubleshooting
> **Map:** [Guide](0-getting-started.md) → **Troubleshooting** → end of the linear guide

This guide covers common issues, error messages, and solutions when working with ZettelTeX.

## 1. Workspace Discovery Error (Exit Code 2)

### Symptom
```
Error de workspace: working directory not found: .../notes/slipbox. Check --workspace-root and the minimal structure (notes/slipbox, projects, template)
```

### Cause
ZettelTeX was executed outside a valid workspace root, or one of the required directories is missing.

### Solution
1. Ensure your terminal working directory is inside the workspace, or specify `--workspace-root`:
   ```bash
   zetteltex --workspace-root /path/to/workspace <command>
   ```
2. If this is a new workspace, initialize the directory structure:
   ```bash
   zetteltex init
   ```
   This creates `notes/slipbox/`, `projects/`, `template/`, and `notes/documents.tex`.

## 2. LaTeX and Rendering Failures

### Symptom
`render`, `render_all`, or `render_updates` fails with an external tool error or missing binary.

### Checks
Verify that the required TeX tools are installed and available in your `PATH`:

```bash
pdflatex --version
biber --version
make4ht --version
```

### Common TeX Compilation Issues
- **Missing packages or custom classes**: Ensure `template/texnote.cls` and `template/style.sty` are in place and accessible by LaTeX.
- **Bibliography errors**: If `biber` fails with `Cannot find control file ...`, run `pdflatex` first to generate the `.bcf` file. In ZettelTeX, passing `--biber` automatically performs the required 3-pass sequence (`pdflatex` -> `biber` -> `pdflatex`).
- **Parallel render errors**: If running `render_all` or `render_updates` with multiple workers, ZettelTeX prints a summary of all errors at the end of the batch. Check the listed files and fix any syntax errors in individual `.tex` files.

## 3. Broken or Unresolved References

### Symptom
Cross-references appear as `[??]` in rendered PDFs, or `validate_references` reports issues.

### Diagnosis
Run validation across your workspace:

```bash
zetteltex synchronize
zetteltex validate_references
```

The output identifies:
- `missing_note`: A link (`\excref`, `\exref`, `\exhyperref`) or `\transclude` points to a note that does not exist in `notes/slipbox/`.
- `missing_label`: The target note exists, but the specified `\label{...}` tag was not found.

### Solution
1. Check for typos in note names or label tags.
2. Run `synchronize` to ensure `slipbox.db` has indexed recent edits:
   ```bash
   zetteltex synchronize
   ```
3. If referencing across notes, verify that `notes/documents.tex` contains `\externaldocument[<note>-]{<note>}`. You can re-add missing entries with:
   ```bash
   zetteltex addtodocuments <note-name>
   ```

## 4. Editor Command (`edit`) Not Working

### Symptom
`zetteltex edit <note>` fails with `No editor configured` or fails to launch your editor.

### Solution
1. Run interactive configuration to set your preferred editor:
   ```bash
   zetteltex init_config
   ```
   Or set `editor` directly in `zetteltex.toml`:
   ```toml
   [general]
   editor = "code"   # or "vim", "nvim", "nano", or full executable path
   ```
2. Ensure the configured editor binary is available in your `PATH`.

### Related
For details on available fuzzy TUI actions and keybindings, see the [Fuzzy Search guide](4-fuzzy-search.md).

## 5. PDF Viewer Issues

### Symptom
Opening PDFs from fuzzy search fails with `Could not open the PDF with any candidate viewer`.

### Solution
1. Ensure a supported PDF viewer is installed (`qpdfview`, `zathura`, `okular`, `evince`, or `xdg-open`).
2. Alternatively, configure your preferred viewer via the `ZETTELTEX_PDF_OPENER` environment variable:
   ```bash
   export ZETTELTEX_PDF_OPENER="zathura"
   ```
3. Note that `qpdfview` is launched with `--unique` by default to reuse existing windows.

## 6. Fuzzy Search in Non-Interactive Shells

### Symptom
Running `zetteltex fuzzy` in a script or non-TTY terminal fails or attempts to launch an external terminal emulator.

### Solution
- Use `--inline` to run a pure text search loop in the current terminal stream:
  ```bash
  zetteltex fuzzy --inline
  ```
- Or use `--action` for programmatic / scripted lookups:
  ```bash
  zetteltex fuzzy --action copy-exhyperref --query "compactness"
  ```

## 7. Cleaning Stale or Orphan Artifacts

### Symptom
Deleted or renamed notes still leave orphan `.pdf` or `.md` files in export directories.

### Solution
Run the cleanup command:
```bash
zetteltex clean
```
This safely scans your export directories and deletes `.pdf` and `.md` files that are no longer tracked in `slipbox.db`.

## Next steps

This concludes the User Guide. For complete syntax of every command, see the [Command Reference](../reference/commands.md). To review the setup from the beginning, return to [Getting Started](0-getting-started.md).

## See Also

* [Testing Strategy](../architecture/testing.md) — what behavior is verified and how.
* [Exit Codes](../reference/exit-codes.md) — return codes used in scripts and CI.
