# API para desarrollo

Este documento resume los puntos de extension y modulos internos relevantes de la implementación en Rust.

Navegación recomendada:

- [Funciones de codigo (índice)](../05-funciones-codigo/README.md)
- [Guía tecnica (índice)](README.md)
- [Catálogo por comando](../03-comandos/README.md)

## crates/zetteltex-cli

Archivo principal:
- crates/zetteltex-cli/src/main.rs

Responsabilidades:
- Parseo de comandos con `clap`.
- Orquestación de operaciones de notas/proyectos/render/sync/export.
- Gestión de errores y códigos de salida.

## crates/zetteltex-db

Archivo principal:
- crates/zetteltex-db/src/lib.rs

Responsabilidades:
- Migraciones SQLite.
- CRUD y consultas de soporte para la CLI.
- Estado de render (last_build_date_pdf / last_edit_date).

## crates/zetteltex-parser

Archivo principal:
- crates/zetteltex-parser/src/lib.rs

Responsabilidades:
- Parsear labels, citas y referencias en notas.
- Parsear inclusiones de proyectos (transclude).

## crates/zetteltex-core

Archivo principal:
- crates/zetteltex-core/src/lib.rs

Responsabilidades:
- Validación de paths del workspace.
- Tipos base compartidos.

## Estrategia de pruebas

Archivo principal:
- crates/zetteltex-cli/tests/cli_smoke.rs

Incluye pruebas de:
- comando inválido
- códigos de salida
- sincronización
- render y biber
- exportación
- utilidades
- manejo de errores

Ver también:

- [Arquitectura](../04-arquitectura/README.md)
- [Índice maestro](../00-indice/README.md)
