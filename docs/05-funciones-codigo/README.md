# Funciones de codigo (indice)

Criterio de seleccion en esta fase:
- funciones de entrada por comando,
- funciones de sincronizacion y validacion,
- funciones de render/export de alto impacto,
- funciones de persistencia base,
- funciones de parseo y validacion de workspace.

## zetteltex-cli

- [run_command](crates/zetteltex-cli/run_command.md)
- [synchronize_notes](crates/zetteltex-cli/synchronize_notes.md)
- [synchronize_projects](crates/zetteltex-cli/synchronize_projects.md)
- [validate_references](crates/zetteltex-cli/validate_references.md)
- [render_updates_cmd](crates/zetteltex-cli/render_updates_cmd.md)
- [load_zetteltex_config](crates/zetteltex-cli/load_zetteltex_config.md)
- [rename_file](crates/zetteltex-cli/rename_file.md)

## zetteltex-db

- [init_database](crates/zetteltex-db/init_database.md)
- [upsert_note](crates/zetteltex-db/upsert_note.md)
- [replace_project_inclusions](crates/zetteltex-db/replace_project_inclusions.md)
- [notes_needing_render](crates/zetteltex-db/notes_needing_render.md)

## zetteltex-parser

- [parse_note](crates/zetteltex-parser/parse_note.md)
- [parse_project_inclusions](crates/zetteltex-parser/parse_project_inclusions.md)

## zetteltex-core

- [WorkspacePaths::discover](crates/zetteltex-core/workspacepaths_discover.md)
- [WorkspacePaths::validate](crates/zetteltex-core/workspacepaths_validate.md)

## Enlaces relacionados

- [Catalogo por comando](../03-comandos/README.md)
- [Guia tecnica](../02-guia-tecnica/README.md)

Atajos por crate:

- CLI: [run_command](crates/zetteltex-cli/run_command.md)
- DB: [init_database](crates/zetteltex-db/init_database.md)
- Parser: [parse_note](crates/zetteltex-parser/parse_note.md)
- Core: [WorkspacePaths::discover](crates/zetteltex-core/workspacepaths_discover.md)

Ver tambien:

- [API para desarrollo](../02-guia-tecnica/api.md)
- [Indice maestro](../00-indice/README.md)
