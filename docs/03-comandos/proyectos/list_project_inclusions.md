# list_project_inclusions

## Proposito
Mostrar notas incluidas en un proyecto segun transclude.

## Sintaxis
`zetteltex --workspace-root <workspace> list_project_inclusions <project>`

## Parametros
- project: nombre del proyecto.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> list_project_inclusions algebra
```

## Comandos relacionados
- [list_note_projects](list_note_projects.md)
- [synchronize](../sync/synchronize.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ListProjectInclusions)
- Prepaso: synchronize_notes + synchronize_projects
