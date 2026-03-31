# Inicio rapido detallado

Este recorrido te lleva de cero a un flujo funcional con zetteltex.

Siguiente nota recomendada: [Flujo diario](flujo-diario.md)

## 1. Verifica herramientas base

```bash
cargo --version
pdflatex --version
biber --version
```

Si `pdflatex` o `biber` no estan disponibles, revisa [Solucion de problemas](solucion-problemas.md).

## 2. Instala o compila la CLI

Opcion A: compilacion local

```bash
cargo build --release -p zetteltex-cli
./target/release/zetteltex --help
```

Opcion B: instalacion en PATH

```bash
cargo install --path crates/zetteltex-cli --force
zetteltex --help
```

Comandos y sintaxis: [Referencia de comandos](../03-comandos/00-referencia.md)

## 3. Prepara un workspace valido

El `--workspace-root` debe apuntar a una carpeta con esta estructura minima:

```text
<workspace>/
  notes/slipbox/
  projects/
  template/
```

Configuracion y detalles: [Configuracion](configuracion.md)

## 4. Crea tu primera nota

```bash
zetteltex --workspace-root <workspace> newnote espacio_metrico
```

Resultados esperados:
- se crea `notes/slipbox/espacio_metrico.tex`,
- se actualiza `notes/documents.tex`,
- se inserta la nota en `slipbox.db`.

## 5. Edita y sincroniza

```bash
zetteltex --workspace-root <workspace> edit espacio_metrico
zetteltex --workspace-root <workspace> synchronize
zetteltex --workspace-root <workspace> validate_references
```

## 6. Renderiza PDF

```bash
zetteltex --workspace-root <workspace> render espacio_metrico
```

Con bibliografia (`biber`):

```bash
zetteltex --workspace-root <workspace> render espacio_metrico pdf true
```

## 7. Exporta a Markdown

```bash
zetteltex --workspace-root <workspace> export_markdown espacio_metrico
zetteltex --workspace-root <workspace> export_all_markdown
```

Detalles de salida: [Exportacion Markdown](exportacion.md)

## 8. Navega con fuzzy

```bash
zetteltex --workspace-root <workspace> fuzzy
zetteltex --workspace-root <workspace> fuzzy --inline
```

Atajos y comportamiento: [Fuzzy search](fuzzy.md)

## 9. Proximos pasos

- Para el ciclo operativo completo, sigue [Flujo diario](flujo-diario.md).
- Para errores frecuentes, usa [Solucion de problemas](solucion-problemas.md).
- Para una ruta segun perfil, abre [Rutas de lectura](../00-indice/rutas-lectura.md).
