# Pipeline de export

## Problema que resuelve
Transformar notas y proyectos a Markdown en rutas de salida configurables.

## Flujo general
1. Se carga configuracion de export desde `zetteltex.toml`.
2. Se selecciona alcance (nota/proyecto/todo).
3. Se generan archivos Markdown y metadatos asociados.
4. Se escriben salidas en carpetas configuradas.

## Configuracion relevante
Bloque `[export]` en `zetteltex.toml`:
- `obsidian_vault`
- `notes_subdir`
- `projects_subdir`

## Comandos del pipeline
- `export_markdown`
- `export_project_markdown`
- `export_all_notes_markdown`
- `export_all_projects_markdown`
- `export_all_markdown`
- `to_md`

## Buenas practicas
- ejecutar `synchronize` antes de exportar,
- usar `export_all_markdown` para publicacion completa.

## Componentes involucrados
- `crates/zetteltex-cli/src/main.rs`
- `crates/zetteltex-db/src/lib.rs`

## Comandos relacionados
- [export_all_markdown](../03-comandos/export/export_all_markdown.md)
- [export_project_markdown](../03-comandos/export/export_project_markdown.md)
- [Configuracion](../01-guia-usuario/configuracion.md)

## Lectura siguiente
- [Sincronizacion](sincronizacion.md)
