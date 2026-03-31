# Inicio Rapido

Guia practica para usar zetteltex desde la CLI en Rust.

Navegacion recomendada:

- [Guia de usuario (indice)](README.md)
- [Catalogo por comando](../03-comandos/README.md)
- [Solucion de problemas](solucion-problemas.md)
- [Indice maestro](../00-indice/README.md)

## 1. Verificacion Minima

```bash
cargo --version
pdflatex --version
biber --version
```

## 2. Ayuda General

```bash
cargo build --release -p zetteltex-cli
./target/release/zetteltex --workspace-root . --help
```

Si prefieres usarlo desde PATH:

```bash
cargo install --path crates/zetteltex-cli --force
zetteltex --workspace-root . --help
```

## 3. Crear Tu Primera Nota

```bash
./target/release/zetteltex --workspace-root . newnote espacio_metrico
```

Esto crea:
- notes/slipbox/espacio_metrico.tex
- referencia en notes/documents.tex
- registro en slipbox.db

## 4. Editar

Con nombre explicito:

```bash
./target/release/zetteltex --workspace-root . edit espacio_metrico
```

Sin argumento (abre la nota mas reciente):

```bash
./target/release/zetteltex --workspace-root . edit
```

## 5. Sincronizar y Validar Referencias

```bash
./target/release/zetteltex --workspace-root . synchronize
./target/release/zetteltex --workspace-root . validate_references
```

## 6. Render PDF

```bash
./target/release/zetteltex --workspace-root . render espacio_metrico
```

Con `biber`:

```bash
./target/release/zetteltex --workspace-root . render espacio_metrico pdf true
```

## 7. Exportar Markdown

```bash
./target/release/zetteltex --workspace-root . export_markdown espacio_metrico
./target/release/zetteltex --workspace-root . export_all_markdown
```

## 8. Flujo Diario Recomendado

```bash
./target/release/zetteltex --workspace-root . newnote tema_nuevo
./target/release/zetteltex --workspace-root . edit tema_nuevo
./target/release/zetteltex --workspace-root . synchronize
./target/release/zetteltex --workspace-root . render tema_nuevo
```
