# Fuzzy search

Interfaz fuzzy nativa de la CLI.

Navegacion recomendada:

- [Comando fuzzy (detalle)](../03-comandos/fuzzy/fuzzy.md)
- [Guia de usuario](README.md)
- [Funciones de codigo](../05-funciones-codigo/README.md)
- [Indice maestro](../00-indice/README.md)

## Uso recomendado

- Usar `zetteltex fuzzy` para abrir la sesion en terminal nueva.
- Usar `zetteltex fuzzy --inline` para ejecutar la sesion en la terminal actual.

`--inline` usa TUI nativa con busqueda en vivo, lista de resultados y panel de preview.

Atajos disponibles en TUI:
- Enter / Ctrl+H: copiar `\\exhyperref[...]` y cerrar.
- Ctrl+R: copiar `\\excref[...]` y cerrar.
- Ctrl+E: abrir en editor y cerrar.
- Ctrl+O: abrir PDF y cerrar.
- Ctrl+N: crear nota desde la busqueda actual y cerrar.
- Ctrl+P: crear nota desde portapapeles y cerrar.
- Ctrl+Alt+N: atajo alternativo para crear desde portapapeles.

Persistencia:
- Estado unificado: `.fuzzy_state.json` (historial + cache de popularidad en la raiz del workspace).
