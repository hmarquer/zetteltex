# remove_duplicate_citations

## Proposito
Eliminar citas duplicadas detectadas durante procesamiento de notas.

## Sintaxis
`zetteltex --workspace-root <workspace> remove_duplicate_citations`

## Ejemplo
```bash
zetteltex --workspace-root <workspace> remove_duplicate_citations
```

## Comandos relacionados
- [list_citations](../notas/list_citations.md)
- [validate_references](validate_references.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RemoveDuplicateCitations)
- Funcion principal: remove_duplicate_citations_cmd
