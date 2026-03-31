# export_markdown

## Proposito
Exportar una nota a Markdown usando configuracion de export.

## Sintaxis
`zetteltex --workspace-root <workspace> export_markdown <note>`

## Parametros
- note: nota objetivo.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> export_markdown espacio_metrico
```

## Comandos relacionados
- [to_md](to_md.md)
- [export_all_markdown](export_all_markdown.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ExportMarkdown)
- Funcion principal: export_markdown
