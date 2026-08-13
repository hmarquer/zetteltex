# render

## Proposito
Renderizar una nota o un proyecto, por defecto en PDF.

## Sintaxis
`zetteltex --workspace-root <workspace> render <name> [--project] [--format <pdf|html>] [--biber]`

## Parametros
- name: nombre de la nota o proyecto objetivo.
- --project: fuerza a tratar `name` como proyecto.
- --format: formato de salida, `pdf` o `html` (default `pdf`).
- --biber: fuerza la ejecucion de biber para la bibliografia.

## Deteccion automatica de tipo

Sin `--project`, si `name` existe solo como nota se renderiza la nota; si solo
existe como proyecto, se renderiza el proyecto. Si existe como nota **y** como
proyecto, se avisa y no se renderiza nada; usa `--project` para desambiguar.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> render espacio_metrico
zetteltex --workspace-root <workspace> render espacio_metrico --format pdf --biber
zetteltex --workspace-root <workspace> render espacio_metrico --format html --biber
zetteltex --workspace-root <workspace> render algebra --project
```

## Comandos relacionados
- [render_all](render_all.md)
- [biber](biber.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Render)
- Funcion principal: render_note_cmd / render_project_cmd
