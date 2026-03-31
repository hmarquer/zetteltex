# export_project

## Proposito
Exportar un proyecto a una carpeta de salida.

## Sintaxis
`zetteltex --workspace-root <workspace> export_project <folder> [texfile]`

## Parametros
- folder: carpeta destino.
- texfile: archivo tex opcional.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> export_project salida proyecto.tex
```

## Comandos relacionados
- [export_project_markdown](export_project_markdown.md)
- [export_all_markdown](export_all_markdown.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ExportProject)
- Funcion principal: export_project
