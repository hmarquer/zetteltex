# Exportacion a Markdown

Los comandos de exportacion generan archivos Markdown para notas y proyectos.

Navegacion recomendada:

- [Catalogo por comando](../03-comandos/README.md)
- [Pipeline de export](../02-guia-tecnica/pipeline-export.md)
- [Guia de usuario](README.md)
- [Indice maestro](../00-indice/README.md)

## Comandos

```bash
cargo run -p zetteltex-cli -- --workspace-root . export_markdown <nota>
cargo run -p zetteltex-cli -- --workspace-root . export_project_markdown <proyecto>
cargo run -p zetteltex-cli -- --workspace-root . export_all_notes_markdown
cargo run -p zetteltex-cli -- --workspace-root . export_all_projects_markdown
cargo run -p zetteltex-cli -- --workspace-root . export_all_markdown
```

## Directorios de Salida

- Notas: jabberwocky/latex/zettelkasten
- Proyectos: jabberwocky/latex/asignaturas

## Contenido Generado

Para notas:
- frontmatter (si aplica)
- link y embed a PDF
- referencias a otras notas
- etiquetas detectadas por keywords

Para proyectos:
- frontmatter con titulo y tags
- link y embed a PDF
- lista de notas incluidas
- etiquetas detectadas

## Recomendaciones

- Ejecutar `synchronize` antes de exportar para asegurar consistencia.
- Ejecutar `export_all_markdown` para publicacion completa.
