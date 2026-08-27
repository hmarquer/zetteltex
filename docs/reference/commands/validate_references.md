# `zetteltex validate_references`

Scans the workspace for broken cross-references and missing transclusion targets, reporting each issue found.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] validate_references [OPTIONS]
```

---

## Options & Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--notes-only` | flag | `false` | Validate only cross-references in `notes/slipbox/`. |
| `--projects-only` | flag | `false` | Validate only `\transclude` targets in `projects/`. |

---

## Behavior & Internal Workflow

1. Unless `--projects-only` is given, `synchronize_notes` runs first so the database reflects current note metadata.
2. `validate_references` queries the database for broken links across the selected scope.
3. If no issues are found, prints "All references are valid" and exits `0`.
4. Otherwise, prints each issue in the form `- [<kind>] <source> -> <target_note> [<target_label>]` and exits `1`.

### Issue Types

| Kind | Meaning |
|---|---|
| `missing_note` | A link (`\excref`, `\exref`, `\exhyperref`) or `\transclude` points to a note that does not exist in `notes/slipbox/`. |
| `missing_label` | The target note exists, but the specified `\label{...}` tag was not found. |

---

## Exit Codes

* **`0`**: All references in scope are valid.
* **`1`**: One or more broken references were detected.
* **`2`**: Workspace discovery error.

---

## Examples

```bash
# Validate all cross-references and transclusions
zetteltex validate_references

# Check only note cross-references
zetteltex validate_references --notes-only

# Check only project transclusions
zetteltex validate_references --projects-only
```

---

## See Also

* [`synchronize`](synchronize.md) — Refresh the database before validation.
* [`addtodocuments`](addtodocuments.md) — Repair missing `\externaldocument` entries.
