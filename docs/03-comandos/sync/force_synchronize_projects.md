# force_synchronize_projects

## Proposito
Forzar sincronizacion de proyectos e inclusiones.

## Sintaxis
`zetteltex --workspace-root <workspace> force_synchronize_projects`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> force_synchronize_projects
```

## Comandos relacionados
- [synchronize](synchronize.md)
- [force_synchronize_notes](force_synchronize_notes.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ForceSynchronizeProjects)
- Funcion principal: synchronize_projects
