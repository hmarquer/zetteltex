# biber_project

## Proposito
Ejecutar biber para un proyecto.

## Sintaxis
`zetteltex --workspace-root <workspace> biber_project <name> [folder]`

## Parametros
- name: proyecto objetivo.
- folder: carpeta opcional del artefacto.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> biber_project algebra
```

## Comandos relacionados
- [render_project](render_project.md)
- [biber](biber.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::BiberProject)
- Funcion principal: run_biber_project_cmd
