# Visión General del Workspace

La aplicación se organiza como un workspace de Cargo con crates especializados para separar responsabilidades.

```text
zetteltex/
├── crates/
│   ├── zetteltex-cli/
│   ├── zetteltex-core/
│   ├── zetteltex-db/
│   └── zetteltex-parser/
├── notes/
├── projects/
├── template/
└── slipbox.db
```

## Referencias Crates
- **`zetteltex-cli`**: Orquestación de comandos y manejo de códigos de salida.
- **`zetteltex-core`**: Descubrimiento y validación de la estructura del workspace.
- **`zetteltex-db`**: Operaciones sobre las tablas de base de datos.
- **`zetteltex-parser`**: Parseo de labels y citas en notas de LaTeX.

Navegación recomendada:
- [Índice de Arquitectura](README.md)
- [Flujo de Comando](02-flujo-comando.md)
