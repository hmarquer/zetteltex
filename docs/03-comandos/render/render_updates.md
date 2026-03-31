# render_updates

## Proposito
Renderizar solo elementos desactualizados segun timestamps de base de datos.

## Sintaxis
`zetteltex --workspace-root <workspace> render_updates [format] [--workers N]`

## Parametros
- format: formato de salida, default pdf.
- --workers N: paralelismo (default interno: 4).

## Ejemplo
```bash
zetteltex --workspace-root <workspace> render_updates pdf --workers 6
```

## Comandos relacionados
- [render_all](render_all.md)
- [synchronize](../sync/synchronize.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RenderUpdates)
- Funcion principal: render_updates_cmd
