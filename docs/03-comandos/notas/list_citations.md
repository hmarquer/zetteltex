# list_citations

## Proposito
Listar citas detectadas en una nota.

## Sintaxis
`zetteltex --workspace-root <workspace> list_citations <name>`

## Parametros
- name: nota a analizar.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> list_citations topologia_general
```

## Comandos relacionados
- [validate_references](../sync/validate_references.md)
- [remove_duplicate_citations](../sync/remove_duplicate_citations.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ListCitations)
- Funcion principal: list_citations
