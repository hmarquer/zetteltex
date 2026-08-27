# ZettelTeX — Audit Action Items

*Working list derived from [`review_report.md`](../review_report.md) (combined security/quality review). Check items off as they are implemented, and update this file in the same session as the change.*

Each item records its severity (per the review), the risk it closes, a starting point in the source, and the verification needed. Items are roughly ordered by priority following the review's recommended roadmap: **safe paths → safe shell-escape → safe symlink handling → process timeouts → dependency/supply-chain hygiene → CI hardening → architectural refactor → new features.**

---

## Phase A — Security (highest priority)

### A1. Make `-shell-escape` opt-in, off by default
- **Severity:** CRITICAL (F1). Arbitrary code execution via `\write18`.
- **[x] Status:** Done
- **Done:** added `allow_shell_escape: bool` (default `false`) to `RenderConfig` (`crates/zetteltex-cli/src/fuzzy.rs`). `pdf.rs` (`run_pdflatex_pass`) and `html.rs` (`render_html_single_pass`) now only add `-shell-escape`/`--shell-escape` when the config enables it, so it is **off by default**. Documented with a security warning in `docs/reference/config-reference.md` (`[render] allow_shell_escape`). Verified: `cargo build`, `cargo test --workspace` (84 passing), `cargo clippy --workspace -- -D warnings` (clean), `cargo fmt --check` (clean).
- **Optionally add later:** a per-invocation CLI `--allow-shell-escape` flag on `render`/`render_all` (not required to close the critical issue).

### A2. Centralize and enforce path-component validation
- **Severity:** HIGH (F2; R1/R2). Path traversal via unsanitized note/project/label names.
- **[x] Status:** Done (entry-point validation; full newtype migration optional)
- **Done:** added `validate_component_name` (single shared helper) to `zetteltex-core` rejecting empty, `.`/`..`, `/`, `\`, and absolute names, with unit tests. Applied it at the **input boundaries** of F2: `create_note`/`create_project` (CLI), `rename_file` (interactive stdin), and the `\transclude{...}` name parsed from note content in `export.rs` (the highest-risk, third-party-content path). Added a CLI smoke test asserting `newnote ../../evil` fails. Verified: `cargo build`, `cargo test --workspace` (87 passing), `cargo clippy --workspace -- -D warnings` (clean), `cargo fmt --check` (clean).
- **Optionally complete later (R1/R2):** wrap in `NoteName`/`ProjectName` newtypes and centralize path construction as `WorkspacePaths::note_tex_path`/`project_dir` returning `Result`, migrating the remaining ~20 internal `.join()` call sites so the guarantee cannot be bypassed.

### A3. Stop following symlinks during project scanning
- **Severity:** HIGH for shared workspaces (F3; R5).
- **[x] Status:** Done
- **Done:** `collect_tex_files()` (`crates/zetteltex-cli/src/sync.rs:338`) now uses `fs::symlink_metadata()`, skips symlinks explicitly, and only recurses into real directories, closing the escape/cycle/DoS surface during sync/validation. Added a unit test (`collect_tex_files_skips_symlinks`) confirming an external `.tex` reachable via symlink is not collected. Verified: `cargo build`, `cargo test --workspace` (88 passing), `cargo clippy --workspace -- -D warnings` (clean), `cargo fmt --check` (clean).
- **Optionally add later:** if symlink support is desired, canonicalize and check the target against the workspace root instead of skipping outright.

### A4. Add a timeout to all external tool invocations
- **Severity:** MEDIUM (F5; R3/R4).
- **[x] Status:** Done
- **Done:** `run_external_tool()` now accepts a configurable timeout. New field `[render] render_timeout_secs` (default `120`) in `RenderConfig` (`crates/zetteltex-cli/src/fuzzy.rs`), exposed via `RenderConfig::tool_timeout()`. The timeout is applied to every external render tool (`pdflatex`, `make4ht`, `biber`) in `pdf.rs`, `html.rs`, and `render/mod.rs`. Implemented time-bounded execution (`run_with_timeout` in `src/util.rs`) using `spawn` + `try_wait` with a deadline, killing the child on expiry instead of blocking forever on `Command::output()`. Documented in `docs/reference/config-reference.md`. Verified: `cargo build`, `cargo test --workspace` (88 passing), `cargo clippy --workspace -- -D warnings` (clean), `cargo fmt --check` (clean).

### A5. Harden mutex handling in parallel renders
- **Severity:** MEDIUM (F4). Mutex poisoning can silently truncate a batch render.
- **[x] Status:** Done
- **Done:** In `render/progress.rs` (`.lock().expect(...)` on the shared work queue): replaced the panicking `expect` with a poison-resilient `queue.lock().unwrap_or_else(|e| e.into_inner())`, so if a lock is poisoned the remaining worker threads keep draining the queue instead of dying and truncating the batch. Additionally wrapped each `job` body in `catch_unwind` (`AssertUnwindSafe`), converting a worker panic into a `RenderEvent::Failed` (with the panic message) rather than silently dropping the item and possibly disconnecting the channel early. Verified: `cargo build`, `cargo test --workspace` (88 passing), `cargo clippy --workspace -- -D warnings` (clean), `cargo fmt --check` (clean).

### A6. Fix regex-replacement `$`-injection in rename
- **Severity:** LOW–MEDIUM (F7; R6). Silent `.tex` corruption on rename.
- **[ ] Status:** Open
- `rename.rs` interpolates user text into `Regex::replace_all` replacement strings; a `$` is reinterpreted as a capture-group reference.
- **Fix:** use `regex::NoExpand(...)` anywhere a dynamic value is used as a replacement (e.g. `\label{{{new_label}}}`).

### A7. Escape TOML string values in interactive config
- **Severity:** LOW (F8).
- **[ ] Status:** Open
- `init_config_interactive` splices user answers into `format!` TOML literals without quote escaping; a `"` yields invalid/altered `zetteltex.toml`.
- **Fix:** escape `"` (and backslashes) in every answer before interpolation.

---

## Phase B — Dependency & supply-chain hygiene

### B1. Add `cargo audit` to CI (and confirm/refute the `anyhow` flag)
- **Severity:** MEDIUM, unconfirmed (F6; §3).
- **[ ] Status:** Open
- `Cargo.lock` pins `anyhow 1.0.102`; ChatGPT flagged RUSTSEC-2026-0190 (unsoundness in `Error::downcast_mut()`, fixed in `>=1.0.103`). Not directly used, but confirm with a real `cargo audit` run; bump `anyhow` if flagged.
- **Action:** install/add `cargo audit` step in `.github/workflows/ci.yml`; run locally to confirm; bump `anyhow` to `>=1.0.103` if the advisory applies.

### B2. Add `cargo deny check` to CI
- **Severity:** tooling gap (§3, §4).
- **[ ] Status:** Open
- No `deny.toml` exists and no license/duplicate policy is enforced automatically. Add a minimal `deny.toml` (MIT workspace licenses) and a `cargo deny check` CI step.

---

## Phase C — CI/CD hardening

### C1. Pin actions by SHA and modernize deprecated actions
- **Severity:** LOW (F9).
- **[ ] Status:** Open
- `.github/workflows/*` use mutable tags (`@v4`, `@v1`); `actions-rs/toolchain@v1` is archived (replace with `dtolnay/rust-toolchain`); `actions/upload-release-asset@v1` is deprecated (replace with `softprops/action-gh-release` or direct `gh release upload`).

### C2. Scope `contents: write` only to the publish job
- **Severity:** LOW (F9).
- **[ ] Status:** Open
- `release.yml` declares `contents: write` at the workflow level; scope it to just the `publish` job so the build job (which compiles third-party code) runs with least privilege.

---

## Phase D — Refactoring

### D1. Static regexes via `LazyLock`
- **Severity:** perf (§5.1; R8).
- **[ ] Status:** Open
- Parser regexes (`\label`, `\cite`, `\ref`, ...) are recompiled per call. Move to module-level `static ... LazyLock<Regex>`.

### D2. Deduplicate single-target vs. batch backlink lookup
- **Severity:** perf/maintainability (§5.8; R7).
- **[ ] Status:** Open
- `notes_referencing_target` and `build_incoming_references_index` both re-parse every note. Extract shared per-file extraction into one function.

### D3. Structured error types at library boundaries
- **Severity:** maintainability (R9).
- **[ ] Status:** Open
- Migrate crate-public boundary APIs from `anyhow::Result` to a structured `thiserror` enum (`ZettelError`) where it improves consumer error handling.

### D4. Reduce zero-copy string overhead in parsing
- **Severity:** perf (§5.2; R10).
- **[ ] Status:** Open
- Hot parsing paths allocate `String`s repeatedly; consider `Cow<'a, str>` / `NoteView<'a>`.

---

## Phase E — New features (speculative / longer-term)

- **[ ] E1.** `zetteltex doctor` — single diagnostic command (reuses `ensure_template_available_or_suggest_init`, `validate_references`, orphan detection, + dangling DB rows).
- **[ ] E2.** Watch mode (`zetteltex watch` / `render_updates --watch`) via a `notify`-based loop calling render-updates on changes.
- **[ ] E3.** Note/link-graph export (`zetteltex graph`) → GraphML/Mermaid/JSON/Canvas from existing DB queries.
- **[ ] E4.** Restricted shell-escape preset (`-shell-restricted` + `texmf.cnf` allowlist) as default "shell escape enabled" mode (complements A1).
- **[ ] E5.** Incremental parser caching (parse results in SQLite keyed by file hash/mtime).
- **[ ] E6.** `zetteltex --safe` render flag bundling hardened defaults (no shell-escape, no external writes, no symlink-following, timeouts, size limits, worker cap).
- **[ ] E7.** Typst export backend. *(Speculative — verify against real render pipeline first.)*
- **[ ] E8.** LSP mode. *(Speculative — see review §0.)*

---

## Already verified as done (no action needed)

- **`unsafe` audit:** zero `unsafe` blocks in the workspace.
- **SQL injection:** all `zetteltex-db` queries use `rusqlite::params![]`; occasional `format!` only interpolates hardcoded identifier literals.
- **Command injection:** all external processes use `Command::new(bin).args([...])`, never a shell string.
- **Panics in production paths:** remaining `.unwrap()`/`.expect()` sites are in tests or compile-time-constant regex literals.
- **Dead `indicatif` dependency:** already removed from the manifests (was §4/#10 in the review).
- **CI baseline:** `fmt` / `clippy` / `test` already run on push/PR; Rust toolchain pinned to current stable.
