# synchronize_notes

## Firma
`fn synchronize_notes(paths: &WorkspacePaths) -> Result<SyncStats>`

## Responsabilidad
Parsear notas `.tex`, persistir metadatos (nota/labels/citations) y reconstruir links de referencias.

## Flujo interno resumido
1. recorre `notes/slipbox`.
2. parsea contenido con `parse_note`.
3. hace `upsert_note` y reemplaza labels/citations.
4. limpia links y los vuelve a construir resolviendo labels objetivo.
5. devuelve estadisticas de sincronizacion.

## Errores y casos borde
- archivos no `.tex` se omiten.
- referencias no resolubles incrementan contador de `unresolved_references`.

## Llamadores principales
- `run_command` en `synchronize`, `force_synchronize*`.

## Relacionado
- [synchronize](../../../03-comandos/sync/synchronize.md)
- [validate_references](validate_references.md)
- [parse_note](../zetteltex-parser/parse_note.md)

## Ubicacion
- `crates/zetteltex-cli/src/main.rs`
