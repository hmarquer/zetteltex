# fuzzy

## Proposito
Abrir la interfaz fuzzy para busqueda y acciones rapidas.

## Sintaxis
`zetteltex --workspace-root <workspace> fuzzy [--inline]`

## Parametros
- --inline: ejecuta la sesion en la terminal actual.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> fuzzy
zetteltex --workspace-root <workspace> fuzzy --inline
```

## Comandos relacionados
- [edit](../notas/edit.md)
- [newnote](../notas/newnote.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Fuzzy)
- Funcion principal: fuzzy_cmd
