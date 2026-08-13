# export_draft

## Proposito
Convertir/volcar un archivo de entrada a un borrador de salida.

## Sintaxis
`zetteltex --workspace-root <workspace> export_draft <input_file> <output_file>`

## Parametros
- input_file: origen.
- output_file: destino.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> export_draft entrada.tex salida.md
```

## Comandos relacionados
- [export_markdown](export_markdown.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::ExportDraft)
- Funcion principal: export_draft
