# addtodocuments

## Proposito
Agregar una nota a notes/documents.tex para referencias cruzadas de LaTeX.

## Sintaxis
`zetteltex --workspace-root <workspace> addtodocuments <name>`

## Parametros
- name: nota a registrar.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> addtodocuments algebra_lineal
```

## Comandos relacionados
- [newnote](newnote.md)
- [synchronize](../sync/synchronize.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::AddToDocuments)
- Funcion principal: add_to_documents
