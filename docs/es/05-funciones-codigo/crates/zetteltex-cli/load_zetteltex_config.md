# load_zetteltex_config

## Firma
`fn load_zetteltex_config(paths: &WorkspacePaths) -> ZetteltexConfig`

## Responsabilidad
Cargar configuración desde `zetteltex.toml` con fallback seguro a defaults.

## Flujo interno resumido
1. localiza archivo de config en root.
2. intenta parseo TOML.
3. si falla, emite warning y retorna defaults.

## Uso principal
- resolucion de settings de render, export y fuzzy.

## Relacionado
- [Configuración](../../../01-guia-usuario/configuracion.md)
- [pipeline-export](../../../02-guia-tecnica/pipeline-export.md)

## Ubicación
- `crates/zetteltex-cli/src/main.rs`
