# API para desarrollo

Este documento resume los puntos de extension y modulos internos relevantes de la implementacion en Rust.

Navegacion recomendada:

- [Funciones de codigo (indice)](../05-funciones-codigo/README.md)
- [Guia tecnica (indice)](README.md)
- [Catalogo por comando](../03-comandos/README.md)

## crates/zetteltex-cli

Archivo principal:
- crates/zetteltex-cli/src/main.rs

Responsabilidades:
- Parseo de comandos con `clap`.
- Orquestacion de operaciones de notas/proyectos/render/sync/export.
- Gestion de errores y codigos de salida.

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
- Validacion de paths del workspace.
- Tipos base compartidos.

## Estrategia de pruebas

Archivo principal:
- crates/zetteltex-cli/tests/cli_smoke.rs

Incluye pruebas de:
- comando invalido
- codigos de salida
- sincronizacion
- render y biber
- exportacion
- utilidades
- manejo de errores

Ver tambien:

- [Arquitectura](../04-arquitectura/README.md)
- [Indice maestro](../00-indice/README.md)
