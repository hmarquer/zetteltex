# rename_file

## Firma
`fn rename_file(paths: &WorkspacePaths, old_name: &str, new_name: &str) -> Result<()>`

## Responsabilidad
Renombrar una nota y propagar cambios en referencias cruzadas y DB.

## Flujo interno resumido
1. renombra archivo fisico en `notes/slipbox`.
2. actualiza nombre de nota en DB.
3. reescribe referencias entre notas y documentos.
4. ajusta `notes/documents.tex` cuando aplica.

## Riesgos
- reescritura de patrones en cascada sobre múltiples archivos `.tex`.

## Relacionado
- [rename_note (comando)](../../../03-comandos/notas/rename_note.md)
- [synchronize](../../../03-comandos/sync/synchronize.md)

## Ubicación
- `crates/zetteltex-cli/src/main.rs`
