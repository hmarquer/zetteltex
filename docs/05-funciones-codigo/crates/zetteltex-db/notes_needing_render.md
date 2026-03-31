# notes_needing_render

## Firma
`pub fn notes_needing_render(&self) -> Result<Vec<String>>`

## Responsabilidad
Detectar notas con render PDF pendiente comparando edit/build timestamps.

## Uso principal
- base del comando incremental `render_updates`.

## Relacionado
- [render_updates_cmd](../zetteltex-cli/render_updates_cmd.md)
- [Pipeline de render](../../../02-guia-tecnica/pipeline-render.md)

## Ubicacion
- `crates/zetteltex-db/src/lib.rs`
