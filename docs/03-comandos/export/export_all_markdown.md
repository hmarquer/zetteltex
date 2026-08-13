# export_all_markdown

## Proposito
Exportar notas y proyectos a Markdown en un solo paso.

## Sintaxis
`zetteltex --workspace-root <workspace> export_all_markdown [--notes] [--projects]`

## Parametros
- --notes: exportar solo las notas.
- --projects: exportar solo los proyectos.

Por defecto exporta notas y proyectos. Los flags se pueden combinar.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> export_all_markdown
zetteltex --workspace-root <workspace> export_all_markdown --notes
zetteltex --workspace-root <workspace> export_all_markdown --projects
```

## Comandos relacionados
- [export_markdown](export_markdown.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ExportAllMarkdown)
- Funcion principal: export_all_markdown
