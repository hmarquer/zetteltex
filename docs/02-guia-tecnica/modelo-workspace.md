# Modelo de workspace

## Problema que resuelve
Definir una raiz operativa unica para que todos los comandos trabajen sobre el mismo conjunto de archivos.

## Estructura minima
Dentro de `--workspace-root` deben existir:
- `notes/slipbox`
- `projects`
- `template`

## Como funciona internamente
1. La CLI recibe `--workspace-root` (por defecto `.`).
2. `WorkspacePaths::discover` construye rutas derivadas.
3. `WorkspacePaths::validate` comprueba directorios minimos.
4. Si falta alguno, se devuelve error de workspace y codigo de salida 2.

## Componentes involucrados
- `crates/zetteltex-cli/src/main.rs`
- `crates/zetteltex-core/src/lib.rs`

## Consecuencias operativas
- El comando no depende del directorio actual si `--workspace-root` apunta bien.
- La validacion temprana evita ejecutar operaciones con rutas inconsistentes.

## Comandos relacionados
- [Opcion global --workspace-root](../03-comandos/utilidades/workspace-root.md)
- [Solucion de problemas](../01-guia-usuario/solucion-problemas.md#1-error-de-workspace)

## Lectura siguiente
- [Sincronizacion](sincronizacion.md)
