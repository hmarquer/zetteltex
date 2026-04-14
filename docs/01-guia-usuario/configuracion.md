# Configuracion

La CLI usa un archivo TOML nativo para centralizar la configuracion.

Navegacion recomendada:

- [Guia de usuario (indice)](README.md)
- [Catalogo por comando](../03-comandos/README.md)
- [Guia tecnica](../02-guia-tecnica/README.md)
- [Funciones de codigo](../05-funciones-codigo/README.md)

## Archivo de configuracion

Archivo: `zetteltex.toml` en la raiz del workspace.

Ejemplo:

```toml
[render]
pdf_output_dir = "jabberwocky/latex/pdf"

[export]
obsidian_vault = "jabberwocky"
notes_subdir = "latex/zettelkasten"
projects_subdir = "latex/asignaturas"

[fuzzy]
max_results = 50
history_results = 10
in_refs_weight = 1.5
out_refs_weight = 1.0
selection_color = "#3a3a3a"
state_file = ".fuzzy_state.json"
```

Notas:
- Las rutas relativas se interpretan desde `--workspace-root`.
- Si `zetteltex.toml` no existe, se usan defaults equivalentes.
- Si el TOML tiene error de parseo, la CLI cae a defaults y deja warning en logs.

## Workspace Root

Parametro global:

```bash
--workspace-root <ruta>
```

Por defecto se usa el directorio actual.

## Estructura requerida

Dentro de `--workspace-root` deben existir:

- notes/slipbox
- projects
- template

Y normalmente:

- notes/documents.tex
- slipbox.db (se crea si no existe al ejecutar comandos con DB)

Si la estructura requerida no existe, la CLI retorna codigo 2.

## Variables de entorno

- ZETTELTEX_EDITOR

Define el editor preferido para el comando `edit`.

Orden de fallback:
1. ZETTELTEX_EDITOR
2. code
3. xdg-open

## Herramientas Externas

- pdflatex: requerido para render.
- biber: requerido para flujos con bibliografia.
- xclip: requerido para operaciones de portapapeles de fuzzy.
- terminal emulator: requerido para `zetteltex fuzzy` (modo por defecto en terminal nueva).

Orden de preferencia de terminal para fuzzy:
1. alacritty
2. x-terminal-emulator
3. gnome-terminal
4. konsole
5. kitty

Modo inline de fuzzy:
- `zetteltex fuzzy --inline` ejecuta la sesion en la terminal actual.

Archivos de estado de fuzzy:
- `.fuzzy_state.json` (historial + cache de popularidad)
