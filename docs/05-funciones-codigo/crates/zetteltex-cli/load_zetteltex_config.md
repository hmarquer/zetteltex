# load_zetteltex_config

## Firma
`fn load_zetteltex_config(paths: &WorkspacePaths) -> ZetteltexConfig`

## Responsabilidad
Cargar configuracion desde `zetteltex.toml` con fallback seguro a defaults.

## Flujo interno resumido
1. localiza archivo de config en root.
2. intenta parseo TOML.
3. si falla, emite warning y retorna defaults.

## Uso principal
- resolucion de settings de render, export y fuzzy.

## Relacionado
- [Configuracion](../../../01-guia-usuario/configuracion.md)
- [pipeline-export](../../../02-guia-tecnica/pipeline-export.md)

## Ubicacion
- `crates/zetteltex-cli/src/main.rs`
