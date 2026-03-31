# validate_references

## Firma
`fn validate_references(paths: &WorkspacePaths) -> Result<Vec<ValidationIssue>>`

## Responsabilidad
Comprobar que toda referencia apunte a nota y label existentes.

## Flujo interno resumido
1. recorre notas `.tex`.
2. parsea referencias con `parse_note`.
3. verifica `note_exists`.
4. verifica `label_exists`.
5. devuelve lista de incidencias (`missing_note`, `missing_label`).

## Llamadores principales
- `run_command` en comando `validate_references`.

## Relacionado
- [validate_references (comando)](../../../03-comandos/sync/validate_references.md)
- [synchronize_notes](synchronize_notes.md)

## Ubicacion
- `crates/zetteltex-cli/src/main.rs`
