# export_project_markdown

## Proposito
Exportar un proyecto a Markdown.

## Sintaxis
`zetteltex --workspace-root <workspace> export_project_markdown <project>`

## Parametros
- project: proyecto objetivo.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> export_project_markdown algebra
```

## Comandos relacionados
- [export_project](export_project.md)
- [export_all_projects_markdown](export_all_projects_markdown.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ExportProjectMarkdown)
- Funcion principal: export_project_markdown
