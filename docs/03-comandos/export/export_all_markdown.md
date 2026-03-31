# export_all_markdown

## Proposito
Exportar notas y proyectos a Markdown en un solo paso.

## Sintaxis
`zetteltex --workspace-root <workspace> export_all_markdown`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> export_all_markdown
```

## Comandos relacionados
- [export_all_notes_markdown](export_all_notes_markdown.md)
- [export_all_projects_markdown](export_all_projects_markdown.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ExportAllMarkdown)
- Funcion principal: export_all_markdown
