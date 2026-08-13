# Referencia de comandos

Todos los comandos usan el binario `zetteltex` y aceptan el parametro global `--workspace-root`.

Para detalle por comando (una nota por comando):
- [Catalogo por comando](README.md)

Para comprender implementacion interna:
- [Guia tecnica](../02-guia-tecnica/README.md)
- [Funciones de codigo](../05-funciones-codigo/README.md)

Compilacion recomendada:

```bash
cargo build --release -p zetteltex-cli
```

Ejemplo base (binario local compilado):

```bash
./target/release/zetteltex --workspace-root . <comando>
```

Alternativa durante desarrollo:

```bash
cargo run -p zetteltex-cli -- --workspace-root . <comando>
```

## Puesta en marcha

- init
- init_config

## Notas

- newnote <name>
- rename_note <name>
- remove_note <name>
- list_recent_files [n]
- list_unreferenced
- rename_recent [n]
- addtodocuments <name>
- list_citations <name>
- edit [name]

## Proyectos

- newproject <name>
- list_projects
- list_project_inclusions <project>
- list_note_projects <note>
- export_project <folder> [texfile]
- export_draft <input_file> <output_file>

## Exportacion Markdown

- export_markdown <name> [--project]
- export_all_markdown [--notes] [--projects]

## Render

- render <name> [--project] [--format <pdf|html>] [--biber]
- render_all [--format <pdf|html>] [--workers N]
- render_all_projects [--format <pdf|html>] [--workers N]
- render_updates [--format <pdf|html>] [--workers N]
- biber <name> [folder] [--project]

Nota: sin `--project`, `render`/`biber`/`export_markdown` detectan si `name`
es nota o proyecto; si `name` existe como ambos, se avisa y no se hace nada.
`--format` acepta `pdf` o `html` (default `pdf`); `--biber` fuerza la
ejecucion de biber.

## Sincronizacion y validacion

- synchronize
- force_synchronize_notes
- force_synchronize_projects
- force_synchronize
- validate_references
- remove_duplicate_citations

## Utilidades

- clean

## Fuzzy

- fuzzy [--inline]

## Codigos de salida

- 0: exito.
- 1: error de ejecucion del comando.
- 2: error de workspace (estructura invalida o faltante).

## Ejemplos rapidos

```bash
./target/release/zetteltex --workspace-root . list_projects
./target/release/zetteltex --workspace-root . fuzzy
./target/release/zetteltex --workspace-root . fuzzy --inline
./target/release/zetteltex --workspace-root . render_all
./target/release/zetteltex --workspace-root . render_all --workers 8
./target/release/zetteltex --workspace-root . render_updates --workers 6
./target/release/zetteltex --workspace-root . render teoria --format html
./target/release/zetteltex --workspace-root . export_markdown 4.1-algebra-conmutativa
```

