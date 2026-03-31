# render_project

## Proposito
Renderizar un proyecto completo.

## Sintaxis
`zetteltex --workspace-root <workspace> render_project <name> [format] [biber]`

## Parametros
- name: proyecto objetivo.
- format: formato de salida, default pdf.
- biber: true/false para bibliografia.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> render_project algebra pdf true
```

## Comandos relacionados
- [render](render.md)
- [biber_project](biber_project.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RenderProject)
- Funcion principal: render_project_cmd
