# Pipeline de render

## Problema que resuelve
Compilar notas y proyectos a PDF con soporte de bibliografia y ejecucion en lote.

## Flujo general
1. El comando resuelve objetivo (nota/proyecto/todos/updates).
2. Se determina formato (`pdf` por defecto).
3. Se invoca `pdflatex` y opcionalmente `biber`.
4. Se actualiza estado de build en base de datos.

## Variantes de pipeline
- `render` para nota individual.
- `render_project` para proyecto individual.
- `render_all` y `render_all_pdf` para lote de notas.
- `render_all_projects` para lote de proyectos.
- `render_updates` para pendientes segun timestamps.

## Parametros operativos
- `--workers N` para paralelismo en lotes.
- bandera booleana de `biber` en comandos de render individual.

## Dependencias externas
- `pdflatex`
- `biber` (solo cuando aplica)

## Componentes involucrados
- `crates/zetteltex-cli/src/main.rs`
- `crates/zetteltex-db/src/lib.rs`

## Comandos relacionados
- [render](../03-comandos/render/render.md)
- [render_updates](../03-comandos/render/render_updates.md)
- [biber_project](../03-comandos/render/biber_project.md)

## Lectura siguiente
- [Pipeline de export](pipeline-export.md)
