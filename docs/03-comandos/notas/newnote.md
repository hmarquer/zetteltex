# newnote

## Proposito
Crear una nueva nota en notes/slipbox y registrarla en la base de datos.

## Sintaxis
`zetteltex --workspace-root <workspace> newnote <name>`

## Parametros
- name: nombre de la nota (sin extension .tex).

## Ejemplo
```bash
zetteltex --workspace-root <workspace> newnote espacio_metrico
```

## Errores frecuentes
- workspace invalido.
- nombre ya existente.

## Comandos relacionados
- [edit](edit.md)
- [synchronize](../sync/synchronize.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Newnote)
- Funcion principal: create_note
