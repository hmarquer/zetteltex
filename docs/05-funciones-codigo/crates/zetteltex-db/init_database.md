# init_database

## Firma
`pub fn init_database(database_path: &Path) -> Result<Database>`

## Responsabilidad
Crear/abrir la base SQLite y aplicar migraciones necesarias.

## Flujo interno resumido
1. llama `Database::open`.
2. configura pragmas base.
3. ejecuta `migrate`.
4. retorna handle listo para uso.

## Relacionado
- [Modelo de datos](../../../02-guia-tecnica/modelo-datos.md)
- [synchronize_notes](../zetteltex-cli/synchronize_notes.md)

## Ubicacion
- `crates/zetteltex-db/src/lib.rs`
