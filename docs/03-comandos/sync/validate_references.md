# validate_references

## Proposito
Validar referencias entre notas y reportar enlaces rotos.

## Sintaxis
`zetteltex --workspace-root <workspace> validate_references`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> validate_references
```

## Comandos relacionados
- [synchronize](synchronize.md)
- [remove_duplicate_citations](remove_duplicate_citations.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ValidateReferences)
- Funcion principal: validate_references
