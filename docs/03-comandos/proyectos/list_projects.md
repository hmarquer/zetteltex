# list_projects

## Proposito
Listar proyectos conocidos por la base de datos.

## Sintaxis
`zetteltex --workspace-root <workspace> list_projects`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> list_projects
```

## Comandos relacionados
- [newproject](newproject.md)
- [list_project_inclusions](list_project_inclusions.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ListProjects)
- Datos: init_database + db.list_projects
