# newproject

## Proposito
Crear un nuevo proyecto en projects/ con su archivo .tex base.

## Sintaxis
`zetteltex --workspace-root <workspace> newproject <name>`

## Parametros
- name: nombre del proyecto.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> newproject teoria_de_grafos
```

## Comandos relacionados
- [list_projects](list_projects.md)
- [render_project](../render/render_project.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Newproject)
- Funcion principal: create_project
