# rename_label

## Proposito
Renombrar una etiqueta dentro de una nota.

## Sintaxis
`zetteltex --workspace-root <workspace> rename_label <note> <old_label> <new_label>`

## Parametros
- note: nota objetivo.
- old_label: etiqueta previa.
- new_label: nueva etiqueta.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> rename_label analisis def:old def:new
```

## Comandos relacionados
- [rename_file](rename_file.md)
- [validate_references](../sync/validate_references.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RenameLabel)
- Funcion principal: rename_label
