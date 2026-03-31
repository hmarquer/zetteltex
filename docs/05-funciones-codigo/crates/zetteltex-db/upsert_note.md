# upsert_note

## Firma
`pub fn upsert_note(&self, filename: &str, last_edit_date: DateTime<Utc>) -> Result<i64>`

## Responsabilidad
Insertar o actualizar una nota por filename preservando semantica idempotente.

## Flujo interno resumido
1. inserta fila si no existe.
2. en conflicto por `filename`, actualiza `last_edit_date`.
3. recupera y retorna `note_id`.

## Relacionado
- [synchronize_notes](../zetteltex-cli/synchronize_notes.md)
- [validate_references](../zetteltex-cli/validate_references.md)

## Ubicacion
- `crates/zetteltex-db/src/lib.rs`
