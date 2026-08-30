# Reference
> **Map:** [Guide](../guide/0-getting-started.md) → **Reference** → [Architecture](../architecture/overview.md)

Comprehensive technical reference for the ZettelTeX command-line interface, configuration parameters, global options, and process exit codes.

---

## Contents

| Document | Description |
|---|---|
| [**Command Reference**](commands.md) | Index of all subcommands, grouped by category, each linking to a dedicated page with full syntax, options, behavior, exit codes, and examples. |

## Individual Command Pages

Each subcommand has its own page under `commands/`, covering synopsis, arguments/options, internal workflow, exit codes, and examples:

* [Workspace Init](commands/init.md) · [Config](commands/init_config.md)
* [Notes](commands/newnote.md) · [Projects](commands/newproject.md)
* [Render](commands/render.md) · [Render All](commands/render_all.md) · [Render Updates](commands/render_updates.md)
* ... and more. See the [Command Reference index](commands.md) for the complete list.

## Other Reference Documents

| Document | Description |
|---|---|
| [**Configuration Reference**](config-reference.md) | Full specification of `zetteltex.toml` fields across `[general]`, `[render]`, `[export]`, and `[fuzzy]`. |
| [**Exit Codes**](exit-codes.md) | Definition of return codes (`0`, `1`, `2`) and guidelines for scripting / CI integration. |
| [**Global Options**](global-options.md) | Binary-wide flags (`--workspace-root`, `--help`, `--version`) and environment variables. |

---

## Quick Navigation

* **Need a quick tutorial?** See the [User Guide](../guide/0-getting-started.md).
* **Looking for architecture details?** See [Architecture Overview](../architecture/overview.md).
* **Looking for function signatures?** See [Code Reference](../internals/functions.md).

## See Also

* Up: [User Guide](../guide/0-getting-started.md) — start here if you have never used ZettelTeX.
* Down: [Internals / zetteltex-cli](../internals/zetteltex-cli.md) — command dispatch implementation.
