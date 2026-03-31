# export_all_notes_markdown

## Proposito
Exportar todas las notas a Markdown.

## Sintaxis
`zetteltex --workspace-root <workspace> export_all_notes_markdown`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> export_all_notes_markdown
```

## Comandos relacionados
- [export_all_projects_markdown](export_all_projects_markdown.md)
- [export_all_markdown](export_all_markdown.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ExportAllNotesMarkdown)
- Funcion principal: export_all_notes_markdown
