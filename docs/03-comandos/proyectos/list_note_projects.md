# list_note_projects

## Proposito
Listar proyectos donde aparece una nota via inclusion.

## Sintaxis
`zetteltex --workspace-root <workspace> list_note_projects <note>`

## Parametros
- note: nombre de la nota.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> list_note_projects topologia_general
```

## Comandos relacionados
- [list_project_inclusions](list_project_inclusions.md)
- [synchronize](../sync/synchronize.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ListNoteProjects)
- Prepaso: synchronize_notes + synchronize_projects
