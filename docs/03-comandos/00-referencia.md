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

## Notas

- newnote <name>
- rename_file <old> <new>
- rename_label <note> <old_label> <new_label>
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
- to_md <note>

## Exportacion Markdown

- export_markdown <note>
- export_project_markdown <project>
- export_all_notes_markdown
- export_all_projects_markdown
- export_all_markdown

## Render

- render <name> [format] [biber]
- render_project <name> [format] [biber]
- render_all [format] [--workers N]
- render_all_pdf [--workers N]
- render_all_projects [format] [--workers N]
- render_updates [format] [--workers N]
- biber <name> [folder]
- biber_project <name> [folder]

## Sincronizacion y validacion

- synchronize
- force_synchronize_notes
- force_synchronize_projects
- force_synchronize
- validate_references
- remove_duplicate_citations

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
./target/release/zetteltex --workspace-root . render_all_pdf
./target/release/zetteltex --workspace-root . render_all_pdf --workers 8
./target/release/zetteltex --workspace-root . render_updates --workers 6
./target/release/zetteltex --workspace-root . export_project_markdown 4.1-algebra-conmutativa
```

Ver tambien:

- [Guia de usuario](../01-guia-usuario/README.md)
- [Indice maestro](../00-indice/README.md)
