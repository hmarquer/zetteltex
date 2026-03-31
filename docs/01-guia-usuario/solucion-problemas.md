# Solucion de problemas

Este documento cubre incidencias frecuentes en uso diario.

## 1. Error de workspace

Sintoma comun:
- `Error de workspace: directorio de trabajo no encontrado: ...`

Causa:
- `--workspace-root` apunta a una carpeta sin estructura minima.

Verifica que existan:
- `notes/slipbox`
- `projects`
- `template`

Comando de ejemplo:

```bash
zetteltex --workspace-root /ruta/a/mi-workspace --help
```

Contexto adicional: [Configuracion](configuracion.md)

## 2. Fallo de render (`pdflatex` o `biber`)

Sintomas comunes:
- fallo en `render` o `render_project`,
- error al activar bibliografia.

Comprobaciones:

```bash
pdflatex --version
biber --version
```

Si faltan herramientas, instalalas y repite el render.

Comandos relacionados: [Referencia de comandos](../03-comandos/00-referencia.md#render)

## 3. Referencias invalidas

Sintoma:
- `validate_references` reporta notas/labels faltantes.

Flujo sugerido:

```bash
zetteltex --workspace-root <workspace> synchronize
zetteltex --workspace-root <workspace> validate_references
```

Si persiste:
- revisa que la nota objetivo exista,
- revisa etiquetas `\\label{...}`,
- revisa sintaxis de referencias.

## 4. Exportacion incompleta en Markdown

Sintoma:
- faltan archivos exportados o rutas inesperadas.

Revisa:
- `zetteltex.toml` en raiz del workspace,
- valores de `[export]` (`obsidian_vault`, `notes_subdir`, `projects_subdir`).

Detalle de exportacion: [Exportacion Markdown](exportacion.md)

## 5. Problemas en `fuzzy`

Sintomas comunes:
- no abre terminal nueva,
- no hay comportamiento esperado en sesiones interactivas.

Acciones:
- prueba `fuzzy --inline` en la terminal actual,
- verifica emulador de terminal disponible,
- revisa estado en `.fuzzy_state.json`.

Guia asociada: [Fuzzy search](fuzzy.md)

## 6. El comando `edit` no abre el editor esperado

Configura variable de entorno:

```bash
export ZETTELTEX_EDITOR=code
```

Luego ejecuta:

```bash
zetteltex --workspace-root <workspace> edit
```

Mas contexto: [Configuracion](configuracion.md#variables-de-entorno)
