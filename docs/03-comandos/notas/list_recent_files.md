# list_recent_files

## Proposito
Listar notas recientes, con limite configurable.

## Sintaxis
`zetteltex --workspace-root <workspace> list_recent_files [n]`

## Parametros
- n: cantidad maxima (default interno: 10).

## Ejemplo
```bash
zetteltex --workspace-root <workspace> list_recent_files 20
```

## Comandos relacionados
- [rename_recent](rename_recent.md)
- [edit](edit.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ListRecentFiles)
- Funcion principal: list_recent_files
