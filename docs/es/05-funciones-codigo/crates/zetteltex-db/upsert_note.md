# upsert_note

## Firma
`pub fn upsert_note(&self, filename: &str, title: &str, last_edit_date: DateTime<Utc>) -> Result<i64>`

## Responsabilidad
Insertar o actualizar una nota por filename preservando semántica idempotente.

## Flujo interno resumido
1. Inserta fila si no existe.
2. En conflicto por `filename`, actualiza `title` y `last_edit_date`.
3. Recupera y retorna `note_id`.

## Relacionado
- [synchronize_notes](../zetteltex-cli/synchronize_notes.md)
- [validate_references](../zetteltex-cli/validate_references.md)

## Ubicación
- `crates/zetteltex-db/src/lib.rs`
