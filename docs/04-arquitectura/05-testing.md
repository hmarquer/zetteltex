# Estrategia de Testing

La validación para asegurar la estabilidad del CLI de Rust y la integridad del Zettelkasten.

## Pruebas de Integración

La suite principal se localiza en el crate del CLI:
- `crates/zetteltex-cli/tests/cli_smoke.rs`

## Cobertura Validada

Las pruebas (`smoke tests`) cubren:
- Casos de éxito (Happy Path) para creación y persistencia.
- Manejo propio de errores esperados.
- Verificación correcta de los códigos de salida (`exit codes`).
- Comportamientos ante la ausencia de dependencias externas (ej. `pdflatex`).
- Ejecución de comandos en batch y otras utilidades.

Navegación recomendada:
- [Índice de Arquitectura](README.md)
