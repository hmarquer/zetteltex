# init_config

## Proposito
Generar el archivo `zetteltex.toml` de forma interactiva en la raiz del
workspace.

## Sintaxis
`zetteltex --workspace-root <workspace> init_config`

## Comportamiento
- Si `zetteltex.toml` ya existe, pregunta antes de sobrescribirlo.
- Pide los valores por seccion (idioma de la interfaz, directorios de salida
  PDF/HTML, vault de Obsidian, parametros fuzzy). Pulsar Enter mantiene el
  valor por defecto.

## Ejemplo
```bash
zetteltex --workspace-root <workspace> init_config
```

## Comandos relacionados
- [init](init.md)
- [Configuracion](../01-guia-usuario/configuracion.md)

## Implementacion
- Despacho: crates/zetteltex-cli/src/main.rs (Commands::InitConfig)
- Funcion principal: init_config_interactive (crates/zetteltex-cli/src/workspace.rs)
