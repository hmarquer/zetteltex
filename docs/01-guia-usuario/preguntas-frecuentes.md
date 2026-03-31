# Preguntas frecuentes

## 1. Necesito compilar siempre en release?

No. Puedes usar:

```bash
cargo run -p zetteltex-cli -- --workspace-root <workspace> <comando>
```

Para uso continuo, suele ser mas rapido usar binario compilado o instalado.

## 2. Que diferencia hay entre `synchronize` y `force_synchronize`?

- `synchronize`: actualiza estado normal.
- `force_synchronize`: reprocesa de forma forzada.

Sintaxis exacta: [Referencia de comandos](../03-comandos/00-referencia.md)

## 3. Cuando usar `render_updates`?

Cuando quieres renderizar solo notas/proyectos con cambios pendientes.

Ejemplo:

```bash
zetteltex --workspace-root <workspace> render_updates --workers 6
```

## 4. Donde se guarda el estado de fuzzy?

En `.fuzzy_state.json` dentro del workspace.

Mas detalles: [Fuzzy search](fuzzy.md)

## 5. Puedo usar zetteltex fuera del root del workspace?

Si, mientras pases `--workspace-root` correcto.

Ejemplo:

```bash
zetteltex --workspace-root /ruta/a/mi-workspace list_projects
```

## 6. Cual es el mejor orden para evitar errores?

Flujo recomendado:
1. editar contenido,
2. `synchronize`,
3. `validate_references`,
4. `render` o `render_updates`,
5. `export_all_markdown` cuando publiques.

Recorrido completo: [Flujo diario](flujo-diario.md)
