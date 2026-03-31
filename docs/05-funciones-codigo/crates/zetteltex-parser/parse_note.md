# parse_note

## Firma
`pub fn parse_note(content: &str) -> Result<ParsedNote>`

## Responsabilidad
Extraer labels, citations y referencias estructuradas desde contenido LaTeX de nota.

## Flujo interno resumido
1. compila regexes de labels/citas/referencias.
2. recorre capturas y llena `ParsedNote`.
3. normaliza claves detectadas con trim.

## Casos contemplados
- `\\label{...}`
- `\\currentdoc{...}`
- `\\cite...{...}` con multiples keys
- `\\excref`, `\\exhyperref`, `\\exref`

## Relacionado
- [synchronize_notes](../zetteltex-cli/synchronize_notes.md)
- [validate_references](../zetteltex-cli/validate_references.md)

## Ubicacion
- `crates/zetteltex-parser/src/lib.rs`
