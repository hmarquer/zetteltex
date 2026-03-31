# rename_file

## Proposito
Renombrar un archivo de nota y actualizar referencias asociadas.

## Sintaxis
`zetteltex --workspace-root <workspace> rename_file <old> <new>`

## Parametros
- old: nombre actual.
- new: nuevo nombre.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> rename_file vieja_nota nueva_nota
```

## Comandos relacionados
- [rename_label](rename_label.md)
- [validate_references](../sync/validate_references.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RenameFile)
- Funcion principal: rename_file
