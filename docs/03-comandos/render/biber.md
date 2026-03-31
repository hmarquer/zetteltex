# biber

## Proposito
Ejecutar biber para una nota concreta.

## Sintaxis
`zetteltex --workspace-root <workspace> biber <name> [folder]`

## Parametros
- name: nota objetivo.
- folder: carpeta opcional del artefacto.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> biber espacio_metrico
```

## Comandos relacionados
- [render](render.md)
- [biber_project](biber_project.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Biber)
- Funcion principal: run_biber_cmd
