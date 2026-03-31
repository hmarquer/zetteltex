# synchronize

## Proposito
Sincronizar notas y proyectos contra la base de datos.

## Sintaxis
`zetteltex --workspace-root <workspace> synchronize`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> synchronize
```

## Comandos relacionados
- [validate_references](validate_references.md)
- [force_synchronize](force_synchronize.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Synchronize)
- Funciones principales: synchronize_notes, synchronize_projects
