# Modelo de Datos

El sistema centraliza el metadato del Zettelkasten utilizando SQLite (`slipbox.db`).

## Tablas Principales

- `note`: Almacena información de notas individuales.
- `project`: Almacena agrupaciones o documentos compuestos.
- `label`, `link`, `citation`, `inclusion`: Tablas relaciones o de metadatos extraídos por el parser.
- `tag`, `notetag`: Manejo de etiquetas asigandas a notas y proyectos.

## Render Incremental

Para evitar recompilaciones innecesarias, la base de datos almacena marcas de tiempo clave:

- `note.last_edit_date` vs `note.last_build_date_pdf`
- `project.last_edit_date` vs `project.last_build_date_pdf`

Si la fecha de edición es más reciente que la de compilación, el elemento se marca como "desactualizado".

Navegación recomendada:
- [Proceso de Renderizado](04-render-actualizacion.md)
- [Módulo DB (Funciones)](../05-funciones-codigo/README.md)
