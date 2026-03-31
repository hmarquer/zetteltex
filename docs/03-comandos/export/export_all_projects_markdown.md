# export_all_projects_markdown

## Proposito
Exportar todos los proyectos a Markdown.

## Sintaxis
`zetteltex --workspace-root <workspace> export_all_projects_markdown`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> export_all_projects_markdown
```

## Comandos relacionados
- [export_all_notes_markdown](export_all_notes_markdown.md)
- [export_all_markdown](export_all_markdown.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ExportAllProjectsMarkdown)
- Funcion principal: export_all_projects_markdown
