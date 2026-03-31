# parse_project_inclusions

## Firma
`pub fn parse_project_inclusions(content: &str) -> Result<Vec<Inclusion>>`

## Responsabilidad
Detectar inclusiones `\\transclude` en archivos de proyecto.

## Flujo interno resumido
1. procesa linea a linea.
2. elimina comentarios LaTeX con `strip_latex_comments`.
3. extrae `tag` opcional y `note_filename`.
4. retorna lista de inclusiones.

## Relacionado
- [synchronize_projects](../zetteltex-cli/synchronize_projects.md)
- [Pipeline de sincronizacion](../../../02-guia-tecnica/sincronizacion.md)

## Ubicacion
- `crates/zetteltex-parser/src/lib.rs`
