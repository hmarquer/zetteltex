# `zetteltex export`
> **Map:** [Command Reference](../commands.md) → **`zetteltex export`** → [Internals / CLI](../../internals/zetteltex-cli.md) — implementation

Exports a note or project into a single, self-contained `.tex` file. The output is independent of the notes base: the engine templates (`ztxbase.sty`, `style.sty`) and the class (`texnote.cls` / `texbook.cls`) are inlined into the preamble, every `\transclude` is expanded recursively, and only the citations actually used are embedded. It can be compiled anywhere with a TeX installation.

---

## Synopsis

```bash
zetteltex [--workspace-root <PATH>] export <name> --output <path.tex> [--project]
```

---

## Arguments

| Argument | Type | Required | Default | Description |
|---|---|---|---|---|
| `<name>` | string | Yes | — | Note or project name. |
| `--output <path>` | string | Yes | — | Output `.tex` file. Relative paths resolve against the workspace root; parent directories are created automatically. |
| `--project` | flag | No | — | Force treating `<name>` as a project. |

---

## Behavior & Internal Workflow

1. **Resolution** — `<name>` is resolved against `notes/slipbox/<name>.tex` (note) or `projects/<name>/<name>.tex` (project); `--project` forces project resolution. If the name matches both, `--project` is required.
2. **Preamble** — a new `\documentclass` is derived from the project class `\LoadClass` directive (`article` for notes, `report` for projects). The class body is inlined after stripping its class-only lines, `\RequirePackage{../../template/ztxbase}` is replaced by the inlined `ztxbase.sty`, and `\usepackage{../../template/style}` inside it is replaced by the inlined `style.sty`. `\title`, `\author` and `\date` are copied from the original preamble.
3. **Transclusion expansion** — every `\transclude[tag]{note}` in the body is replaced by the `%<*tag>...%</tag>` block of the target note, expanding nested transcludes recursively. A cycle is reported as an error; each transcluded note gets a single `\phantomsection\label{<note>-note}` anchor so references behave as usual.
4. **References** — `\exref`, `\excref` and `\exhyperref` are redefined: if the target label `<note>-<tag>` is defined in the generated file, they link as before; otherwise they print the note's name in `\texttt`.
5. **Citations** — if any `\cite`/`\parencite`/`\textcite`/`\autocite`/… command is present, only the referenced entries are extracted from `bibliography.bib` and embedded through `\begin{filecontents*}{\jobname.bib}`, followed by `\addbibresource{\jobname.bib}`. With no citations, the bibliography resource is dropped and `\printbibliography` is commented out in the body.
6. **Output** — the assembled document is written to `--output`; the path is printed on success.

---

## Exit Codes

* **`0`**: Standalone exported successfully.
* **`1`**: Note/project not found, template missing, transclusion cycle, missing tag, or citation key not present in `bibliography.bib`.
* **`2`**: Workspace discovery error.

## Compiling the generated file

The output is self-contained and compiles anywhere with a TeX Live installation:

```bash
pdflatex -interaction=nonstopmode my-export.tex
biber my-export           # only if the document contains citations
pdflatex -interaction=nonstopmode my-export.tex   # resolve \ref/\cref pointers
pdflatex -interaction=nonstopmode my-export.tex   # settle the label list
```

The first run materializes the embedded bibliography (`filecontents*`) into `my-export.bib`; when there are citations run `biber` and then `pdflatex` twice so cross-references settle.

---

## Examples

```bash
# Export note 'curvas-diferenciables' to notes/curvas-standalone.tex
zetteltex export curvas-diferenciables --output notes/curvas-standalone.tex

# Export project 'topology-course' to a custom location
zetteltex export topology-course --project --output releases/topology.tex
```

---

## See Also

* [`export_markdown`](export_markdown.md) — Export a note or project to Obsidian Markdown.
* [`export_all_markdown`](export_all_markdown.md) — Export all notes and projects to Markdown.
* [Export pipeline](../../architecture/export-pipeline.md) — The different export paths.