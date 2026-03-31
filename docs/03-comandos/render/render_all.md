# render_all

## Proposito
Renderizar todas las notas con concurrencia configurable.

## Sintaxis
`zetteltex --workspace-root <workspace> render_all [format] [--workers N]`

## Parametros
- format: formato de salida, default pdf.
- --workers N: paralelismo (default interno: 4).

## Ejemplo
```bash
zetteltex --workspace-root <workspace> render_all pdf --workers 8
```

## Comandos relacionados
- [render_all_pdf](render_all_pdf.md)
- [render_updates](render_updates.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RenderAll)
- Funcion principal: render_all_notes_cmd
