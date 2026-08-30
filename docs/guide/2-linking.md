# Linking Notes
> **Map:** [Guide](0-getting-started.md) → **Linking Notes** → [Rendering](3-rendering.md)

The zettelkasten method is built on linking notes. ZettelTeX provides custom LaTeX commands that handle cross-references between notes automatically. These commands work seamlessly whether the target is in the same note or in a different one.

## Cross-referencing notes

### `\excref` — Cross-reference a note

```latex
See \excref{topology-basics} for the definition.
```

This renders as a hyperlinked reference to the note `topology-basics`. No label is needed inside the target note — ZettelTeX resolves it automatically via the `\externaldocument` mechanism.

If you want to reference a specific labeled element inside a note (e.g. a definition or theorem), use the optional argument:

```latex
See \excref[defn:compact-space]{topology-basics} for the definition.
```

### `\exref` — Plain cross-reference

```latex
As shown in \exref{topology-basics}.
```

Same as `\excref` but uses a plain `\ref` instead of `\cref` (no automatic type prefix). Supports the same optional argument for specific labels.

### `\exhyperref` — Custom display text

```latex
See \exhyperref{topology-basics}{the compactness note} for details.
```

Renders as a hyperlink with custom display text. The optional argument works the same way:

```latex
See \exhyperref[defn:compact-space]{topology-basics}{the compactness definition} for details.
```

## Embedding note content

### `\transclude` — Embed a note inside another document

```latex
\transclude{topology-basics}
```

Embeds the full content of another note inline, as if you had pasted it. This command is primarily intended for **projects** — it lets you compose a project document from atomic notes without duplicating content.

You can also include only a tagged section:

```latex
% In topology-basics.tex, surround content with tags:
%<*definitions>
\label{defn:compact-space}
A topological space is compact if...
%</definitions>

% In the project:
\transclude[definitions]{topology-basics}
```

## Synchronize

After editing notes and adding links, synchronize the metadata to the database:

```bash
zetteltex synchronize
```

This parses every `.tex` file in `notes/slipbox/` and `projects/`, extracts `\label`, `\ref`, `\cite`, and `\transclude` commands, and updates the SQLite database. Run `synchronize` after any significant edit so that cross-references and `\transclude` resolve correctly.

## Validate references

To catch broken cross-references:

```bash
zetteltex validate_references
```

Reports any `\excref`, `\exref`, or `\transclude` that points to a missing note or label. Run this after `synchronize` to verify that all links are valid before rendering.

## Next step

After synchronizing, learn how to [render](3-rendering.md) your documents to PDF or HTML.

## See Also

* [Reference / `synchronize`](../reference/commands/synchronize.md) — keeping the database in sync.
* [Sync Process](../architecture/sync-process.md) — how links are discovered and stored.
