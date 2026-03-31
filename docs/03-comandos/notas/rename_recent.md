# rename_recent

## Proposito
Renombrar la nota reciente numero n segun orden de recencia.

## Sintaxis
`zetteltex --workspace-root <workspace> rename_recent [n]`

## Parametros
- n: posicion de la nota reciente (default interno: 1).

## Ejemplo
```bash
zetteltex --workspace-root <workspace> rename_recent 1
```

## Comandos relacionados
- [list_recent_files](list_recent_files.md)
- [rename_file](rename_file.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RenameRecent)
- Funcion principal: rename_recent
