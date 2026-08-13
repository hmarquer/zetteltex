# render_all

## Proposito
Renderizar todas las notas con concurrencia configurable.

## Sintaxis
`zetteltex --workspace-root <workspace> render_all [--format <pdf|html>] [--workers N]`

## Parametros
- --format: formato de salida, `pdf` o `html` (default `pdf`).
- --workers N: paralelismo (default interno: 4).

## Ejemplo
```bash
zetteltex --workspace-root <workspace> render_all --format pdf --workers 8
zetteltex --workspace-root <workspace> render_all --format html --workers 8
```

## Comandos relacionados
- [render](render.md)
- [render_updates](render_updates.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RenderAll)
- Funcion principal: render_all_notes_cmd
