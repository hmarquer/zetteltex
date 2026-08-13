# export_markdown

## Proposito
Exportar una nota o proyecto a Markdown usando configuracion de export.

## Sintaxis
`zetteltex --workspace-root <workspace> export_markdown <name> [--project]`

## Parametros
- name: nombre de la nota o proyecto objetivo.
- --project: fuerza a tratar `name` como proyecto.

## Deteccion automatica de tipo

Sin `--project`, si `name` existe solo como nota se exporta la nota; si solo
existe como proyecto, se exporta el proyecto. Si existe como nota **y** como
proyecto, se avisa y no se exporta nada; usa `--project` para desambiguar.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> export_markdown espacio_metrico
zetteltex --workspace-root <workspace> export_markdown algebra --project
```

## Comandos relacionados
- [export_all_markdown](export_all_markdown.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ExportMarkdown)
- Funcion principal: export_markdown
