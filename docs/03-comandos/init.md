# init

## Proposito
Crear la estructura minima de un workspace y copiar las plantillas LaTeX
necesarias para notas y proyectos.

## Sintaxis
`zetteltex --workspace-root <workspace> init`

## Que crea
- `notes/slipbox/`
- `projects/`
- `template/` con `note.tex`, `project.tex`, `style.sty`, `texbook.cls` y
  `texnote.cls`
- `notes/documents.tex` (si no existe)

## Ejemplo
```bash
zetteltex --workspace-root <workspace> init
```

## Comandos relacionados
- [init_config](init_config.md)
- [Configuracion](../01-guia-usuario/configuracion.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Init)
- Funcion principal: init_workspace (crates/zetteltex-cli/src/workspace.rs)
