# rename_note

## Proposito
Renombrar una nota de forma interactiva: primero el archivo y despues cada una
de sus etiquetas. Cada paso puede omitirse dejando el valor vacio.

## Sintaxis
`zetteltex --workspace-root <workspace> rename_note <name>`

## Parametros
- name: nombre de la nota a renombrar (debe existir en la base de datos).

## Flujo interactivo
1. Pide el nuevo nombre de archivo; si se deja vacio (o es igual al actual),
   el archivo no se renombra.
2. Por cada etiqueta (`\label`) de la nota, pide el nuevo nombre; vacio para
   conservar la etiqueta actual.
3. Si no se renombra ni el archivo ni ninguna etiqueta, avisa
   "No se realizaron cambios".

Al renombrar el archivo, `rename_note` tambien:
- actualiza el nombre en la base de datos,
- reescribe las referencias cruzadas en notas y proyectos,
- ajusta `notes/documents.tex` cuando aplica,
- elimina artefactos exportados obsoletos (pdf/markdown) del nombre viejo.

Al renombrar una etiqueta, actualiza las referencias en notas y proyectos y
resincroniza.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> rename_note espacio_metrico
```

## Comandos relacionados
- [rename_recent](rename_recent.md)
- [validate_references](../sync/validate_references.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::RenameNote)
- Funcion principal: rename_note (crates/zetteltex-cli/src/rename.rs)
- Funciones internas: rename_file, rename_label (mismo archivo)
