# Internals — `zetteltex-core`

> **Map:** [Architecture Overview](../architecture/overview.md) → [Internals](functions.md) → **zetteltex-core** → [Generated rustdoc](https://docs.rs/zetteltex_core)

`zetteltex-core` is the dependency-free value-neutral crate: workspace layout and path validation, plus the runtime i18n macro that both `zetteltex-db` and `zetteltex-cli` depend on. One file of substance (`src/lib.rs`, 108 lines) plus `src/i18n.rs`.

## Public API

| Item | Location | Purpose |
|---|---|---|
| `struct WorkspacePaths` | `lib.rs:19` | The three workspace directories (notes_slipbox, projects, template) plus root |
| `WorkspacePaths::discover(root)` | `lib.rs:28` | Compute paths for a root and validate them; fails if any directory is missing |
| `WorkspacePaths::validate()` | `lib.rs:40` | Ensure `notes/slipbox`, `projects`, `template` all exist (else a descriptive error) |
| `validate_component_name(name)` | `lib.rs:72` | Reject `""`, `.`, `..`, path separators, and absolute paths — the path-traversal guard |
| `Result` / `ZettelError` | `lib.rs:7`/`9` | Error type carrying i18n-ized messages |

```rust
pub struct WorkspacePaths {
    pub root: PathBuf,
    pub notes_slipbox: PathBuf,
    pub projects: PathBuf,
    pub template: PathBuf,
}
```

## Why these two functions carry the whole CLI

Every command (except `init`, which runs before discovery) begins with `WorkspacePaths::discover`. Its `validate()` is what produces the "working directory not found… check `--workspace-root` and the minimal structure" error users see when running the binary outside a workspace. The "Revisa `--workspace-root`" hint is wired into the error string itself ([`lib.rs:56`](../architecture/workspace-model.md) describes the layout this validates).

`validate_component_name` is the security-relevant piece: it closes the path-traversal class for every user-supplied name (notes, projects, rename targets…) before the CLI ever touches the filesystem. Its tests live in the same file (`lib.rs:88`).

## i18n

`src/i18n.rs` exposes `set_lang` and the `tr!` macro (`tr!("ES", "EN", args…)`). The active language comes from config `[general] lang`; only `es` selects Spanish, everything else falls back to English, and reads are lock-free atomics. It is re-exported as `src/i18n.rs` in the CLI as well.

## File map

| Task | Start reading at |
|---|---|
| How a workspace is validated | `crates/zetteltex-core/src/lib.rs` — `discover`, `validate` |
| Guarding a user-supplied name | `crates/zetteltex-core/src/lib.rs` — `validate_component_name` |
| Adding/using translated messages | `crates/zetteltex-core/src/i18n.rs` — `set_lang`, `tr!` |
| The dependency-level contract with `zetteltex-db` | `tr!` macro usage in `crates/zetteltex-db/src/lib.rs` |

---

## See Also

- Up: [Architecture Overview](../architecture/overview.md) — the crate graph
- Down: [Generated rustdoc](https://docs.rs/zetteltex_core) — signatures
- Lateral: [Workspace Model](../architecture/workspace-model.md) — what "a workspace" means on disk