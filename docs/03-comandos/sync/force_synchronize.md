# force_synchronize

## Proposito
Forzar sincronizacion completa de notas y proyectos.

## Sintaxis
`zetteltex --workspace-root <workspace> force_synchronize`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> force_synchronize
```

## Comandos relacionados
- [synchronize](synchronize.md)
- [validate_references](validate_references.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ForceSynchronize)
- Funciones principales: synchronize_notes, synchronize_projects
