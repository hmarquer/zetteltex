# Sincronizacion

## Problema que resuelve
Construir y mantener consistencia entre archivos `.tex`, referencias parseadas e informacion persistida en `slipbox.db`.

## Flujo de notas
1. Recorrer `notes/slipbox`.
2. Parsear labels, citas y referencias.
3. `upsert` de nota por filename.
4. Reemplazar labels y citas.
5. Reconstruir links de referencias entre notas.

## Flujo de proyectos
1. Recorrer subdirectorios en `projects`.
2. Detectar archivo principal `<project>/<project>.tex`.
3. Parsear inclusiones `\\transclude{...}` en tex del proyecto.
4. Resolver nota objetivo y guardar inclusiones.

## Validacion de referencias
- detecta `missing_note`
- detecta `missing_label`

## Componentes involucrados
- `parse_note` y `parse_project_inclusions` en parser
- operaciones de DB en `zetteltex-db`
- orquestacion en `zetteltex-cli`

## Comandos relacionados
- [synchronize](../03-comandos/sync/synchronize.md)
- [force_synchronize](../03-comandos/sync/force_synchronize.md)
- [validate_references](../03-comandos/sync/validate_references.md)

## Lectura siguiente
- [Modelo de datos](modelo-datos.md)
