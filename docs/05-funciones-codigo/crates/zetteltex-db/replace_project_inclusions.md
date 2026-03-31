# replace_project_inclusions

## Firma
`pub fn replace_project_inclusions(&self, project_id: i64, inclusions: &[(i64, String, String)]) -> Result<()>`

## Responsabilidad
Reemplazar atomicamente las inclusiones de un proyecto.

## Flujo interno resumido
1. elimina inclusiones previas del proyecto.
2. inserta inclusiones actuales resueltas.
3. mantiene consistencia con constraints de unicidad.

## Relacionado
- [synchronize_projects](../zetteltex-cli/synchronize_projects.md)
- [list_project_inclusions](../../../03-comandos/proyectos/list_project_inclusions.md)

## Ubicacion
- `crates/zetteltex-db/src/lib.rs`
