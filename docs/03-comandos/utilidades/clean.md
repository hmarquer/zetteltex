# clean

## Proposito
Eliminar archivos pdf y markdown huerfanos de los directorios de exportacion.
Un archivo se conserva si su nombre corresponde a una nota o proyecto registrado
en la base de datos.

## Sintaxis
`zetteltex --workspace-root <workspace> clean`

## Que limpia
- directorios de exportacion de notas y proyectos (configurados en
  `zetteltex.toml`),
- directorios legacy: `markdown/`, `jabberwocky/adjuntos/pdf/` y `pdf/`.

Al final imprime un resumen: `Resumen de limpieza: <N> pdf(s), <M>
markdown(s) eliminado(s)`.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> clean
```

## Comandos relacionados
- [synchronize](../sync/synchronize.md)
- [Configuracion](../../01-guia-usuario/configuracion.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Clean)
- Funcion principal: clean_cmd (crates/zetteltex-cli/src/maintenance.rs)
