# to_md

## Proposito
Exportar una nota concreta a formato Markdown.

## Sintaxis
`zetteltex --workspace-root <workspace> to_md <note>`

## Parametros
- note: nota objetivo.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> to_md espacio_metrico
```

## Comandos relacionados
- [export_markdown](export_markdown.md)
- [export_all_markdown](export_all_markdown.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ToMd)
- Funcion principal: to_md
