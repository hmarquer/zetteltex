# edit

## Proposito
Abrir una nota en editor externo.

## Sintaxis
`zetteltex --workspace-root <workspace> edit [name]`

## Parametros
- name: nota a abrir. Si se omite, intenta la nota reciente.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> edit espacio_metrico
zetteltex --workspace-root <workspace> edit
```

## Errores frecuentes
- editor no disponible en entorno.

## Comandos relacionados
- [newnote](newnote.md)
- [list_recent_files](list_recent_files.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::Edit)
- Funcion principal: edit_cmd
