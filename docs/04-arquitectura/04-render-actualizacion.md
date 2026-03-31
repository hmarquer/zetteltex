# Proceso de Renderizado y Actualización

La herramienta compila el código LaTeX a PDF orquestando llamadas al sistema.

## Características del Renderizado

- **Doble pasada y bibliografía**: Los comandos `render` y `render_project` soportan compilaciones complejas con múltiples pasadas de `pdflatex` y la invocación de `biber` si es necesario.
- **Compilación en lote**: Operaciones como `render_all` y `render_all_projects` iteran sobre toda la colección aplicando eficientemente dos pasadas.

## Actualizaciones Incrementales

El comando `render_updates` selecciona únicamente los elementos desactualizados. Esto se logra comparando los _timestamps_ en la base de datos (ver [Modelo de Datos](03-modelo-datos.md)).

Navegación recomendada:
- [Comando Render](../03-comandos/render/render.md)
- [Comando Render Updates](../03-comandos/render/render_updates.md)
