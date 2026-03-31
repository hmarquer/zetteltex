# remove_note

## Proposito
Eliminar una nota del workspace y de los indices de soporte.

## Sintaxis
`zetteltex --workspace-root <workspace> remove_note <name>`

## Parametros
- name: nota a eliminar.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> remove_note borrador_viejo
```

## Comandos relacionados
- [newnote](newnote.md)
- [list_unreferenced](list_unreferenced.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RemoveNote)
- Funcion principal: remove_note
