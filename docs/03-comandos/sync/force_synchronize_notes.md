# force_synchronize_notes

## Proposito
Forzar sincronizacion de notas.

## Sintaxis
`zetteltex --workspace-root <workspace> force_synchronize_notes`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> force_synchronize_notes
```

## Comandos relacionados
- [synchronize](synchronize.md)
- [force_synchronize_projects](force_synchronize_projects.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ForceSynchronizeNotes)
- Funcion principal: synchronize_notes
