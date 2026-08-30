# Notes and Projects
> **Map:** [Guide](0-getting-started.md) → **Notes and Projects** → [Linking](2-linking.md)

ZettelTeX manages two types of documents: **notes** and **projects**. Understanding the difference is key to using the zettelkasten method effectively.

## Notes

Notes are the core of your zettelkasten. Each note is a single `.tex` file stored in `notes/slipbox/` and should be **atomic**: one idea, one concept, one definition or theorem. This makes notes easy to link, find, and reuse across different contexts.

Examples of notes:

- A definition of a mathematical concept
- A proof technique
- A summary of a research paper
- A key insight from a lecture

### Create a note

```bash
zetteltex newnote compactness-in-metric
```

This does three things:

1. Creates `notes/slipbox/compactness-in-metric.tex` from the `note.tex` template, with the title derived from the filename (e.g. `compactness-in-metric` becomes "Compactness in metric") and the current date set in `\date{}`.
2. Registers the note in `slipbox.db` with the current timestamp.
3. Adds an `\externaldocument` entry to `notes/documents.tex` so other notes can reference it.

The note name must be unique. If a note with that name already exists in the database, the command fails.

## Projects

Projects group multiple notes into a single coherent document. A project is a directory in `projects/` containing its own `.tex` file that pulls in notes via `\transclude`. Projects are useful for longer, structured works that build on atomic notes.

Examples of projects:

- Course notes that combine several related definitions and theorems
- A thesis chapter
- A review article

The key distinction: **notes are the building blocks, projects are the structures built from them.**

> **Tip:** A note and a project can share the same name, but it is not recommended — it forces you to use `--project` every time you want to target the project. If only one exists (note or project), ZettelTeX resolves it automatically without any flag.

### Create a project

```bash
zetteltex newproject topology-course
```

This creates `projects/topology-course/topology-course.tex` from the `project.tex` template — with the title derived from the filename and the current date set in `\date{}` — and registers it in the database.

## Edit a document

```bash
zetteltex edit compactness-in-metric
```

Opens the document in the editor configured during `init_config`. If no name is given, the most recently modified note is opened.

```bash
zetteltex edit topology-course
```

Opens a project instead of a note.

## Next step

Once you have created some notes, learn how to [link them together](2-linking.md) using ZettelTeX's cross-reference commands.

## See Also

* [Reference / `newnote`](../reference/commands/newnote.md) — command syntax.
* [Workspace Model](../architecture/workspace-model.md) — the on-disk layout.
