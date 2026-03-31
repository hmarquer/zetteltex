# render_all_projects

## Proposito
Renderizar todos los proyectos.

## Sintaxis
`zetteltex --workspace-root <workspace> render_all_projects [format] [--workers N]`

## Parametros
- format: formato de salida, default pdf.
- --workers N: paralelismo (default interno: 4).

## Ejemplo
```bash
zetteltex --workspace-root <workspace> render_all_projects pdf --workers 4
```

## Comandos relacionados
- [render_project](render_project.md)
- [render_updates](render_updates.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RenderAllProjects)
- Funcion principal: render_all_projects_cmd
