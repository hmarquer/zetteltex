# Internals — `zetteltex-parser`

> **Map:** [Architecture Overview](../architecture/overview.md) → [Internals](functions.md) → **zetteltex-parser** → [Generated rustdoc](https://docs.rs/zetteltex_parser)

The parser crate is deliberately minimal: a single `lib.rs` (196 lines), 3 data structs, 8 compiled regexes, and 2 public functions. **It is an extractor, not a LaTeX parser.** It pulls structured data out of note and project `.tex` files; it never validates semantics (that happens in the CLI against the database), and it cannot fail — the public functions return plain values, not `Result`.

## Public API

| Function | Signature | Purpose |
|---|---|---|
| `parse_note` | `pub fn parse_note(content: &str) -> ParsedNote` | Extract labels, citations, structured references, plain refs from a note |
| `parse_project_inclusions` | `pub fn parse_project_inclusions(content: &str) -> Vec<Inclusion>` | Extract `\transclude[tag]{note}` pairs from a project's main file |

Supporting types:

```rust
pub struct Reference { pub target_note: String, pub target_label: String }
pub struct ParsedNote { pub labels: Vec<String>, pub citations: Vec<String>,
                        pub references: Vec<Reference>, pub plain_refs: Vec<String> }
pub struct Inclusion  { pub note_filename: String, pub tag: String }
```

## Commands matched

| Command | Field populated | Regex (`lib.rs`) |
|---|---|---|
| `\label{…}` | `labels` | `LABEL_RE` |
| `\currentdoc{…}` | `labels` (alias) | `CURRENTDOC_RE` |
| `\cite{…}` / `\cite*{…}` / `\citealt{…}` / `\cite[preview]{…}` | `citations` (split on `,`, trimmed, empties dropped) | `CITE_RE` |
| `\ref{…}` | `plain_refs` | `REF_RE` |
| `\excref[label]{note}` | `references` | `EXCREF_RE` |
| `\exhyperref[label]{note}{display}` | `references` | `EXHYPERREF_RE` |
| `\exref[label]{note}` | `references` | `EXREF_RE` |
| `\transclude[tag]{note}` / `\transclude{note}` | `Inclusion` | `TRANSCLUDE_RE` |

All outputs are **owned `String`s** (consumers like the DB layer need owned values).

## Comment handling (the interesting part)

LaTeX comments (`%` to end of line) are stripped **before** matching, so commented-out commands are ignored. The subtlety is determining whether a given `%` is escaped.

The parity automaton in `unescaped_comment_index` (`lib.rs:151`):

```rust
fn unescaped_comment_index(line: &str) -> Option<usize> {
    let mut prev_backslash = false;
    for (i, ch) in line.char_indices() {
        if ch == '%' && !prev_backslash {
            return Some(i);
        }
        prev_backslash = ch == '\\' && !prev_backslash;
    }
    None
}
```

A `%` is a real comment when preceded by an **even** number of backslashes; when preceded by an **odd** number it is escaped (`\%`) and the rest of the line survives:

| Input | Behavior |
|---|---|
| `text \% \label{ok}` | `%` escaped → `\label{ok}` is extracted |
| `text \\% \label{bad}\n\label{good}` | `\\%` is a real comment → only `\label{good}` survives (the `\n` rejoin keeps the next line alive) |
| `\transclude{hidden}` after a `\\%` comment | ignored |

`strip_latex_comments` returns a `Cow` — `Borrowed` when there is no comment (zero-alloc fast path), `Owned` only when a line must be cut.

## Two structural asymmetries worth knowing

1. **Whole-content vs line-oriented.** `parse_note` strips comments line-by-line but then operates on the **whole re-joined content**, so a command split across two lines still matches. `parse_project_inclusions` operates **per line** after stripping, so a `\transclude` split across lines does **not** match. This is intentional (transcludes are single-line by convention) but easy to trip on.
2. **`\ex…` optional argument is actually mandatory.** The regexes for `\excref`/`\exref`/`\exhyperref` require the `[label]` argument — `\excred{note}` without brackets does **not** match here. The CLI detects that bracket-less variant with its own regexes (`cli/src/render/mod.rs:752`, `cli/src/rename.rs:252`).

## Known limitations (documented, not bugs)

- No handling of **nested braces**: every group is `[^}]+`, so `\label{foo{bar}}` captures `foo{bar`.
- No `\{`/`\}` escaped-brace handling.
- No positional tracking: only extracted strings are returned, never offsets.
- Regexes are compiled once into `LazyLock` statics — this material "errors-can't-happen" property is why the public API stopped returning `Result` (see `docs/audit-actions.md`).

## What the parser does NOT do (it lives in the CLI)

Reference validation is performed elsewhere, against the database:

- missing note / missing label for `\excref` → `sync.rs check_reference`
- internal broken `\ref` → validated against the file's own labels
- project-local `\ref` → validated across all files of the project (two passes)
- `\transclude` to a nonexistent note → **fatal** during `synchronize`

See [Internals / cli](zetteltex-cli.md) and [Sync Process](../architecture/sync-process.md).

## Tests

`crates/zetteltex-parser/src/lib.rs:164` — three unit tests cover the comment/escape cases above; the integration suite in the CLI (`cli_smoke.rs`) exercises the parser end-to-end through `validate_references`, renames, transclusions, and export. See [Testing Strategy](../architecture/testing.md).

---

## See Also

- Up: [Architecture Overview](../architecture/overview.md) — where parsing fits in the crate graph
- Down: [Generated rustdoc](https://docs.rs/zetteltex_parser) — structs and signatures
- Lateral: [Sync Process](../architecture/sync-process.md) — where `parse_note` feeds the database; [Reference / linking commands](../reference/commands.md)