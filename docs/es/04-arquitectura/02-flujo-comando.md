# Flujo de Comando

El ciclo de vida de la ejecución de una instrucción genérica en `zetteltex` sigue estos pasos:

1. El usuario ejecuta `zetteltex` proporcionando opciones globales como `--workspace-root`.
2. `zetteltex-core` entra en acción para descubrir y validar la estructura base del proyecto.
3. El módulo principal en `zetteltex-cli` (usando `clap`) interpreta el subcommand invocado y despacha la acción.
4. Según la necesidad, se invoca a `zetteltex-parser` para leer archivos LaTeX y extraer metadatos.
5. Se utiliza `zetteltex-db` para consultar o persistir los cambios en la base de datos local `slipbox.db`.
6. En caso de compilación (render), se invocan procesos externos (`pdflatex`, `biber`) en el sistema desde el CLI.

Navegación recomendada:
- [Visión General](01-vision-general.md)
- [Proceso de Renderizado](04-render-actualizacion.md)
