# WorkspacePaths::validate

## Firma
`pub fn validate(&self) -> Result<()>`

## Responsabilidad
Verificar que la estructura mínima requerida exista en disco.

## Validaciones
- `notes/slipbox`
- `projects`
- `template`

## Relacionado
- [WorkspacePaths::discover](workspacepaths_discover.md)
- [Solución de problemas: workspace](../../../01-guia-usuario/solucion-problemas.md#1-error-de-workspace)

## Ubicación
- `crates/zetteltex-core/src/lib.rs`
