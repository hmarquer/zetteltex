# biber

## Proposito
Ejecutar biber para una nota o proyecto concreto.

## Sintaxis
`zetteltex --workspace-root <workspace> biber <name> [folder] [--project]`

## Parametros
- name: nombre de la nota o proyecto objetivo.
- folder: carpeta opcional del artefacto.
- --project: fuerza a tratar `name` como proyecto.

## Deteccion automatica de tipo

Sin `--project`, si `name` existe solo como nota se usa la nota; si solo existe
como proyecto, se usa el proyecto. Si existe como nota **y** como proyecto, se
avisa y no se hace nada; usa `--project` para desambiguar.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> biber espacio_metrico
zetteltex --workspace-root <workspace> biber algebra --project
```

## Comandos relacionados
- [render](render.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Biber)
- Funcion principal: run_biber_cmd
