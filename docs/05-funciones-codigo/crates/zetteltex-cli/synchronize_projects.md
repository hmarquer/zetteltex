# synchronize_projects

## Firma
`fn synchronize_projects(paths: &WorkspacePaths) -> Result<ProjectSyncStats>`

## Responsabilidad
Sincronizar proyectos e inclusiones `\\transclude` en base de datos.

## Flujo interno resumido
1. recorre carpetas en `projects`.
2. detecta `<project>/<project>.tex`.
3. parsea inclusiones en todos los `.tex` del proyecto.
4. resuelve nota destino (`resolve_note_id`).
5. persiste inclusiones con `replace_project_inclusions`.

## Errores y casos borde
- proyectos sin archivo principal se omiten.
- inclusiones con nota no resuelta incrementan `missing_notes`.

## Llamadores principales
- `run_command` en `synchronize`, `force_synchronize*`, listados de proyecto.

## Relacionado
- [list_project_inclusions](../../../03-comandos/proyectos/list_project_inclusions.md)
- [parse_project_inclusions](../zetteltex-parser/parse_project_inclusions.md)
- [replace_project_inclusions](../zetteltex-db/replace_project_inclusions.md)

## Ubicacion
- `crates/zetteltex-cli/src/main.rs`
