# Global Options

ZettelTeX provides global command-line options that apply across all subcommands, as well as environment variables that influence runtime behavior.

---

## Global CLI Options

Global options must be passed to the `zetteltex` binary before or alongside any subcommand.

```bash
zetteltex [GLOBAL_OPTIONS] [SUBCOMMAND] [SUBCOMMAND_OPTIONS]
```

### `--workspace-root <PATH>`

Specifies the root directory of the ZettelTeX workspace.

* **Type**: Path (absolute or relative)
* **Default**: `.` (current working directory)
* **Short flag**: None

```bash
# Execute command on a workspace located in another directory
zetteltex --workspace-root /path/to/my-zettelkasten list_projects

# Render updates in an explicit workspace
zetteltex --workspace-root ~/research/notes render_updates
```

If the specified path does not contain the required workspace structure (`notes/slipbox/`, `projects/`, `template/`), ZettelTeX terminates immediately with [Exit Code 2](exit-codes.md).

> **Note:** The `zetteltex init` command also accepts `--workspace-root` to initialize the directory structure in a custom path.

---

### `-h`, `--help`

Displays help information for the binary or a specific subcommand.

```bash
# View general help and all available subcommands
zetteltex --help

# View help, arguments, and flags for a specific subcommand
zetteltex render --help
zetteltex export_all_markdown --help
```

---

### `-V`, `--version`

Displays the compiled version of ZettelTeX.

```bash
zetteltex --version
```

---

## Default Binary Invocation

Invoking `zetteltex` without any subcommand displays a brief hint message and exits with status `0`:

```bash
$ zetteltex
zetteltex: use --help to see available commands
```

---

## Environment Variables

ZettelTeX reads standard Unix/Linux environment variables to configure fallback behaviors:

| Variable | Description | Fallback Behavior |
|---|---|---|
| `ZETTELTEX_PDF_OPENER` | PDF viewer command for fuzzy `Ctrl+P` | Used to override the auto-detected PDF viewer (`qpdfview`, `zathura`, etc.) |
| `RUST_LOG` | Logging filter level for `tracing` | Defaults to `warn` level; can be set to `info`, `debug`, or `trace` for troubleshooting |
| `PATH` | System executable search path | Used to locate external tools (`pdflatex`, `biber`, `make4ht`, configured editor) |

---

## Related Documentation

* [Command Reference](commands.md) — Comprehensive reference for all subcommands.
* [Configuration Reference](config-reference.md) — `zetteltex.toml` configuration options.
* [Exit Codes](exit-codes.md) — Detailed description of process return codes.
