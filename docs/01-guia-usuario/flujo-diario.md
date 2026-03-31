# Flujo diario

Esta nota describe un flujo operativo estable para trabajo diario.

Nota previa recomendada: [Inicio rapido detallado](inicio-rapido.md)

## Objetivo del flujo

Minimizar errores y mantener sincronizados:
- archivos `.tex`,
- metadata en `slipbox.db`,
- salidas PDF/Markdown.

## Ciclo corto recomendado

```bash
zetteltex --workspace-root <workspace> newnote tema_nuevo
zetteltex --workspace-root <workspace> edit tema_nuevo
zetteltex --workspace-root <workspace> synchronize
zetteltex --workspace-root <workspace> render tema_nuevo
```

## Ciclo con bibliografia

```bash
zetteltex --workspace-root <workspace> render tema_nuevo pdf true
```

Si hay proyecto con inclusiones:

```bash
zetteltex --workspace-root <workspace> render_project nombre_proyecto pdf true
```

## Ciclo de control de calidad

Antes de cerrar una sesion de trabajo:

```bash
zetteltex --workspace-root <workspace> synchronize
zetteltex --workspace-root <workspace> validate_references
zetteltex --workspace-root <workspace> render_updates --workers 4
```

## Ciclo de publicacion

```bash
zetteltex --workspace-root <workspace> export_all_markdown
```

Ver detalles: [Exportacion Markdown](exportacion.md)

## Uso de fuzzy dentro del flujo

Para insertar referencias y abrir archivos rapido:

```bash
zetteltex --workspace-root <workspace> fuzzy --inline
```

Atajos: [Fuzzy search](fuzzy.md)

## Errores operativos comunes

- Workspace invalido: revisa [Solucion de problemas](solucion-problemas.md#1-error-de-workspace).
- Render falla por dependencias: revisa [Solucion de problemas](solucion-problemas.md#2-fallo-de-render-pdflatex-o-biber).
- Referencias rotas: revisa [Solucion de problemas](solucion-problemas.md#3-referencias-invalidas).

## Relacion con referencia de comandos

Para sintaxis exacta de cada comando, usa [Referencia de comandos](../03-comandos/00-referencia.md).
