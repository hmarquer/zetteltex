# Inicio rapido

Guia para poner en marcha un workspace de zetteltex y ejecutar las primeras
operaciones.

## 1. Instalar

Compila e instala el binario:

```bash
cargo install --path crates/zetteltex-cli --force
```

O descarga un binario precompilado: [Instalacion de binario](instalacion-binario.md).

## 2. Crear el workspace

Crea la estructura minima (`notes/slipbox`, `projects`, `template`) y las
plantillas LaTeX:

```bash
mkdir mi_zettelkasten
cd mi_zettelkasten
zetteltex init
```

## 3. Configurar

Genera `zetteltex.toml` interactivamente (directorios de salida, vault de
Obsidian, idioma de la interfaz):

```bash
zetteltex init_config
```

Pulsa Enter para mantener los valores por defecto. Detalle:
[Configuracion](configuracion.md).

## 4. Primeras operaciones

```bash
zetteltex newnote espacio_metrico   # crea una nota
zetteltex edit espacio_metrico      # editala en tu editor
zetteltex synchronize               # actualiza la base de datos
zetteltex render espacio_metrico    # compila a PDF
```

## 5. Siguientes pasos

- Flujo recomendado para el dia a dia: [Flujo diario](flujo-diario.md).
- Sintaxis exacta de cada comando: [Referencia de comandos](../03-comandos/00-referencia.md).
- Solucion de errores: [Solucion de problemas](solucion-problemas.md).
