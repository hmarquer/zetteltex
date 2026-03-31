# Modelo de datos

## Problema que resuelve
Mantener metadatos consistentes de notas/proyectos para sincronizacion, validacion, render incremental y consultas de navegacion.

## Base de datos
Archivo local SQLite: `slipbox.db` en la raiz del workspace.

## Tablas principales
- `note`
- `project`
- `label`
- `link`
- `citation`
- `inclusion`
- `tag`
- `notetag`

## Campos clave para incremental
- `note.last_edit_date`
- `note.last_build_date_pdf`
- `project.last_edit_date`
- `project.last_build_date_pdf`

## Reglas relevantes
- Hay claves foraneas con `ON DELETE CASCADE`.
- Existen restricciones `UNIQUE` para evitar duplicados.
- Se usan operaciones `upsert` para notas y proyectos.

## Componentes involucrados
- `crates/zetteltex-db/src/lib.rs`
- `crates/zetteltex-cli/src/main.rs`

## Consultas usadas por la CLI
- listado de proyectos
- inclusiones por proyecto
- proyectos por nota
- notas no referenciadas
- popularidad para fuzzy

## Comandos relacionados
- [synchronize](../03-comandos/sync/synchronize.md)
- [validate_references](../03-comandos/sync/validate_references.md)
- [list_projects](../03-comandos/proyectos/list_projects.md)

## Lectura siguiente
- [Pipeline de render](pipeline-render.md)
