# Pipeline de render

## Problema que resuelve
Compilar notas y proyectos a PDF o HTML con soporte de bibliografía y ejecución en lote.

## Flujo general
1. El comando resuelve objetivo (nota/proyecto/todos/updates).
2. Se determina formato (`pdf` o `html`, por defecto `pdf`).
3. Se invoca la cadena de compilación correspondiente y opcionalmente `biber`.
4. Se actualiza el estado de build en la base de datos.

## Cadena de compilación por formato

### PDF
- Se ejecutan dos pasadas de `pdflatex` para resolver referencias cruzadas.
- Si se solicita bibliografía (`--biber`), se ejecuta `biber` entre pasadas.

### HTML
- Se ejecuta `make4ht` para convertir LaTeX a HTML.
- Si se solicita bibliografía (`--biber`), se ejecuta `biber` previamente.

## Variantes de pipeline
- `render` para nota o proyecto individual (con `--project` para desambiguar).
- `render_all` para lote de notas.
- `render_all_projects` para lote de proyectos.
- `render_updates` para pendientes según timestamps.

## Render incremental

`render_updates` compara los campos de timestamp en la base de datos:

- PDF: `last_edit_date` vs `last_build_date_pdf`
- HTML: `last_edit_date` vs `last_build_date_html`

Solo se recompilan los elementos cuya edición es más reciente que el último build del formato correspondiente.

## Parámetros operativos
- `--format <pdf|html>` para seleccionar formato de salida.
- `--workers N` para paralelismo en lotes (default: 4).
- `--biber` para forzar la ejecución de biber en comandos de render individual.

## Dependencias externas
- `pdflatex` (formato PDF)
- `biber` (solo cuando aplica bibliografía)
- `make4ht` (formato HTML)

## Componentes involucrados
- `crates/zetteltex-cli/src/main.rs`
- `crates/zetteltex-db/src/lib.rs`

## Comandos relacionados
- [render](../03-comandos/render/render.md)
- [render_updates](../03-comandos/render/render_updates.md)
- [biber](../03-comandos/render/biber.md)

## Lectura siguiente
- [Pipeline de export](pipeline-export.md)
