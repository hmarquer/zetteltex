# render_updates_cmd

## Firma
`fn render_updates_cmd(paths: &WorkspacePaths, format: &str, workers: usize) -> Result<()>`

## Responsabilidad
Renderizar solo notas/proyectos pendientes segun estado incremental en DB.

## Flujo interno resumido
1. consulta notas pendientes (`notes_needing_render`).
2. consulta proyectos pendientes (`projects_needing_render`).
3. ejecuta render paralelo con control de progreso.
4. actualiza estado de build cuando aplica.

## Precondiciones
- base de datos inicializada y sincronizada.
- herramientas externas disponibles para formato objetivo.

## Relacionado
- [render_updates (comando)](../../../03-comandos/render/render_updates.md)
- [notes_needing_render](../zetteltex-db/notes_needing_render.md)
- [Pipeline de render](../../../02-guia-tecnica/pipeline-render.md)

## Ubicacion
- `crates/zetteltex-cli/src/main.rs`
