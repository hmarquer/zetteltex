# render_all_pdf

## Proposito
Renderizar todas las notas en PDF.

## Sintaxis
`zetteltex --workspace-root <workspace> render_all_pdf [--workers N]`

## Parametros
- --workers N: paralelismo (default interno: 4).

## Ejemplo
```bash
zetteltex --workspace-root <workspace> render_all_pdf --workers 6
```

## Comandos relacionados
- [render_all](render_all.md)
- [render_updates](render_updates.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RenderAllPdf)
- Funcion principal: render_all_notes_cmd
