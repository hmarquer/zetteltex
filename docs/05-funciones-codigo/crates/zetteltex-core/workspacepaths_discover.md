# WorkspacePaths::discover

## Firma
`pub fn discover(root: impl Into<PathBuf>) -> Result<Self>`

## Responsabilidad
Derivar rutas de workspace desde root y ejecutar validacion estructural.

## Flujo interno resumido
1. construye `notes_slipbox`, `projects`, `template`.
2. invoca `validate`.
3. retorna `WorkspacePaths` listo para CLI.

## Relacionado
- [Modelo de workspace](../../../02-guia-tecnica/modelo-workspace.md)
- [workspace-root](../../../03-comandos/utilidades/workspace-root.md)

## Ubicacion
- `crates/zetteltex-core/src/lib.rs`
