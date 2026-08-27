# Project Improvements

Planned and in-progress improvements for ZettelTeX. Each area describes the problem, the motivation, and a concrete proposal. Items are checked off as they are implemented.

---

## 1. Note/Project render duplication in `render.rs` (1417 lines — the largest file in the repo)

- [x] **Stage 1 — Done:** Introduced `enum RenderTarget { Note(String), Project(String) }` with `source_dir()`, `source_path()`, `contains_citations()`, `run_biber()`, `prepare_pdf_input()`, and `prepare_html_input()`. Unified the PDF orchestration into a single `render_pdf(paths, target, with_biber)` and the HTML pass into `render_html_single_pass(paths, target)`. Removed the divergent `project_contains_citations` regex — **both** notes and projects now detect citations via `parse_note`, closing the divergence risk. Also removed the two residual `unwrap()` calls in `render.rs` (see item #5).
- [ ] **Open:** `render.rs` is still a single 1400+-line file; see item #2 for the proposed split into submodules.

Originally, `render_note_pdf` / `render_project_pdf` and `render_note_html_single_pass` / `render_project_html_single_pass` repeated almost line-for-line the multi-pass orchestration (pdflatex → biber → pdflatex → pdflatex). Citation detection was duplicated via **two different mechanisms**:

```rust
fn note_contains_citations(...) -> Result<bool> {
    let parsed = parse_note(&content)?;      // uses the real parser
    Ok(!parsed.citations.is_empty())
}

fn project_contains_citations(...) -> Result<bool> {
    let cite_re = Regex::new(r"\\(?:no)?cite[a-zA-Z\*]*\s*(?:\[[^\]]*\]\s*)?\{")?;  // ad-hoc regex
    Ok(cite_re.is_match(&content))
}
```

That was a real divergence risk: if the parser changes what counts as a citation (`\citeauthor`, `\citeyear`, etc.), the project path never learns about it. Now both paths share `RenderTarget::contains_citations()`, which always uses `parse_note`.

---

## 2. Split `render.rs` into submodules

- [x] **Status:** Done

1417 lines in a single file mix: multi-pass orchestration, terminal progress (`render_compact_progress_line`, `build_progress_line_layout`, `terminal_columns`...), SQLite lock retries, and PDF/HTML logic.

**Implemented:** split into `render/{pdf.rs, html.rs, progress.rs, engine.rs}` with a `mod.rs` that re-exports the `pub(crate)` items via `pub(crate) use <sub>::*`. `mod.rs` keeps the command entry points, `RenderTarget`, `PreparedRenderInput`, and the shared helpers (`render_motor`, `render_pass_count`, `ztx_temp_dir`, incoming-references index, inject/referenced-in, biber commands). The PDF orchestration (incl. per-target input preparation) lives in `pdf.rs`, the HTML pass + overrides in `html.rs`, the parallel progress UI in `progress.rs`, and the SQLite lock retry in `engine.rs`. The internal API is unchanged; verified with `cargo build`, `cargo test --workspace` (84 passing), and `cargo clippy --workspace` (clean).

---

## 3. `tracing` declared but almost unused

- [x] **Status:** Done

`tracing` was a workspace dependency initialized in `main()`, but only appeared in 5 call sites (`main.rs`, `fuzzy.rs`) versus dozens of `println!`/`eprintln!` scattered across the CLI.

**Decision (Ruta 1):** `println!`/`eprintln!` are the designed user-facing UX output (they respect i18n via `tr()`), so they are kept as-is. `tracing` is used consistently for *internal diagnostics*:

- `error!` in `main()` for workspace-discovery and command errors.
- `warn!` in `fuzzy.rs` for config/history parse failures.
- `warn!` in `render/engine.rs` for SQLite lock retry backoff.
- `warn!` in `render/progress.rs` for the per-file detail of failed renders (the `N | errores: M` summary stays as UX `println!`).

The subscriber in `main()` filters at `warn` level, so diagnostics go to stderr with levels/targets while UX goes through `println!`/`eprintln!`. Verified with `cargo build`, `cargo test --workspace` (84 passing), and `cargo clippy --workspace` (clean).

---

## 4. Incomplete CI

- [x] **Status:** Done

`.github/workflows/` only had `release.yml` (multi-platform build on release) — nothing ran `cargo test` or `cargo clippy` on push/PR, even though `tests/cli_smoke.rs` is ~92 KB with hundreds of assertions. That was the highest effort/benefit gap: silent regressions before a release.

**Implemented:** added `.github/workflows/ci.yml` running `cargo fmt --check`, `cargo test --workspace`, and `cargo clippy --workspace -- -D warnings` on every push to `main` and every pull request. `cargo fmt --check` was initially left out because the repo was not `fmt`-clean; the codebase was subsequently formatted with `cargo fmt` so the check now passes and is enforced in CI.

---

## 5. Residual `unwrap()` in non-test code

- [x] **Status:** Done

Both `unwrap()` calls in `render.rs` (lines 1290 and 1319):

```rust
let file_name = project_path.file_name().unwrap().to_string_lossy();
```

were replaced with `.context("project file name")?` inside `RenderTarget::prepare_pdf_input` / `prepare_html_input`. No `unwrap()` remains in `render.rs` (the only non-test gap flagged in the report; the remaining `unwrap()` calls in `fuzzy.rs` are inside `#[test]` code).

---

## 6. `MEJORAS.md` as a closed history, not a TODO

- [ ] **Status:** Already addressed (file removed)

Originally all 8 entries were marked `(HECHA)`; it was a refactor changelog, not a list of pending work. As a root-level file it could mislead a new contributor expecting work to do.

**Current state:** the file no longer exists in the repo — it was removed during the English-canonical documentation rewrite. This file (`docs/improvéments.md`) now serves as the living TODO list, leaving the repo root for README/licensing.

---

## 7. i18n limitation in `clap` help

- [ ] **Status:** Open

Documented in the old `MEJORAS.md` ("la ayuda de clap queda en español, límite del derive estático"), but not reflected in the README. If the project aims to be truly bilingual, it is worth evaluating clap's `Command::mut_arg` / dynamic `about` generation based on `lang`, or at least warning explicitly in the README that `--help` is only in Spanish/one language even though runtime messages respect the configured language.

---
