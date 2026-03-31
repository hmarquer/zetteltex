# render

## Proposito
Renderizar una nota, por defecto en PDF.

## Sintaxis
`zetteltex --workspace-root <workspace> render <name> [format] [biber]`

## Parametros
- name: nota objetivo.
- format: formato de salida, default pdf.
- biber: true/false para bibliografia.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> render espacio_metrico
zetteltex --workspace-root <workspace> render espacio_metrico pdf true
```

## Comandos relacionados
- [render_project](render_project.md)
- [biber](biber.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Render)
- Funcion principal: render_note_cmd
