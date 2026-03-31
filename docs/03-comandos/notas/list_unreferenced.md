# list_unreferenced

## Proposito
Mostrar notas sin referencias entrantes o uso detectado.

## Sintaxis
`zetteltex --workspace-root <workspace> list_unreferenced`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> list_unreferenced
```

## Comandos relacionados
- [list_recent_files](list_recent_files.md)
- [synchronize](../sync/synchronize.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ListUnreferenced)
- Funcion principal: list_unreferenced
