# zetteltex - Documentacion Principal

## ¿Qué es ZettelTeX?
ZettelTeX es una herramienta de terminal (CLI) escrita en Rust diseñada para gestionar una base de conocimiento estilo Zettelkasten basada en documentos de LaTeX. Permite administrar, interconectar y compilar de manera eficiente un conjunto de notas, ideas y conceptos individuales escritos en formato `.tex` (conocidos como _zettels_).

## ¿Para qué sirve?
ZettelTeX automatiza las tareas tediosas asociadas con el mantenimiento de grandes colecciones de apuntes, documentos académicos u hojas de notas interconectadas:
- **Gestión ágil de notas**: Crear (`newnote`) o editar (`edit`) notas en LaTeX desde la terminal.
- **Trazabilidad y validación**: Extrae automáticamente todas las etiquetas (`\label`), referencias cruzadas (`\ref`) y citas bibliográficas (`\cite`), permitiéndote detectar referencias rotas o notas huérfanas al instante (`validate_references`).
- **Compilación eficiente a PDF**: Genera archivos PDF (`render`) a partir de tus notas y proyectos utilizando `pdflatex` y `biber`. Solo recompila lo que detecta que ha cambiado, ahorrando tiempo mediante un sistema de caché.
- **Exportación**: Podrás convertir todo tu grafo de conocimiento o notas individuales a archivos Markdown (`export_markdown`) para usarlos con otras herramientas (como Obsidian) o ser publicados en la web.

## ¿Cómo funciona en términos generales?
La herramienta se apoya en una estructura de carpetas específica (conocida como _workspace_) que contiene, entre otros elementos, directorios para `notes/`, `projects/` y un `template/`. Su lógica de funcionamiento interno se compone de:

1. **Un motor de base de datos (`slipbox.db`)**: ZettelTeX utiliza una base de datos local embebida en SQLite para indexar metadatos sobre todas tus notas, proyectos, etiquetas y relaciones de citas. Así sabe exactamente qué documentos dependen de otros.
2. **Sincronización bajo demanda**: Al ejecutar `synchronize`, ZettelTeX escanea internamente las modificaciones en los archivos y parsera el texto LaTeX para actualizar la base de datos de referencias.
3. **Múltiples sub-módulos (Crates)**: La arquitectura en Rust se divide responsabilidades claras: un parser nativo para tokens LaTeX (`zetteltex-parser`), gestión directa de persistencia con SQLite (`zetteltex-db`), las reglas del sistema de ficheros y validación de directorios (`zetteltex-core`) y una amigable interfaz de línea de comandos orquestadora (`zetteltex-cli`).

---

## 🚀 Guía de Inicio Rápido

La mejor forma de entender ZettelTeX es usándolo.

### 1. Requisitos Previos

Asegúrate de tener instalados:
- **Rust y Cargo**: Para compilar e instalar la herramienta (`cargo --version`).
- **LaTeX (pdflatex, biber)**: Para la compilación de documentos PDF (`pdflatex --version`).

### 2. Instalación

Para instalar ZettelTeX globalmente en tu sistema desde el código fuente, ejecuta:

```bash
cargo install --path crates/zetteltex-cli --force
```

De esta forma, puedes llamar al comando `zetteltex` desde cualquier directorio. (Si prefieres no instalarlo, puedes sustituir `zetteltex` por `cargo run --release -p zetteltex-cli --` en los siguientes comandos).

### 3. Crear el Workspace

ZettelTeX necesita una estructura de carpetas mínima para funcionar (con directorios como `notes/`, `projects/` y un `template/`). Puedes automatizar su creación con el comando `init`:

```bash
mkdir mi_zettelkasten
cd mi_zettelkasten
zetteltex init
```
Esto creará la estructura mínima (`notes/slipbox`, `projects`, `template`) y copiará al workspace los archivos reales de plantilla del proyecto (`note.tex`, `project.tex`, `style.sty`, `texbook.cls`, `texnote.cls`).

### 4. Configuración Interactiva

Para personalizar el comportamiento de ZettelTeX en este workspace (directorios de exportación PDF, integración con Obsidian y ajustes visuales de búsqueda), puedes generar un archivo `zetteltex.toml` de manera interactiva:

```bash
zetteltex init_config
```
La terminal te hará una serie de breves preguntas. Puedes pulsar  `Enter` para aceptar los valores por defecto. Si el archivo ya existe, te preguntará si deseas sobrescribirlo de forma segura.

### 5. Uso Básico y Flujo de Trabajo

El flujo principal de ZettelTeX gira alrededor de la terminal y tu editor de texto favorito. Todo debe ejecutarse desde la raíz de tu _workspace_ (o pasar `--workspace-root .`).

#### A. Crear una Nota
```bash
zetteltex newnote espacio_metrico
```
Esto creará automáticamente un archivo en `notes/slipbox/espacio_metrico.tex`, lo registrará en la base de datos e insertará los imports necesarios en tu documento principal.

#### B. Editar la Nota
```bash
zetteltex edit espacio_metrico
```
Abre la nota en tu editor configurado (por ejemplo, Vim, Neovim, o VS Code). Si ejecutas solo `zetteltex edit`, se abrirá automáticamente tu nota más reciente.

#### C. Sincronizar y Revisar
Tras hacer cambios o interconectar tus notas (usando etiquetas `\label` y referencias `\ref` o `\cite`), actualiza la base de datos interna de dependencias:
```bash
zetteltex synchronize
zetteltex validate_references
```
Esto asegurará que todas las referencias entre tus *zettels* estén intactas.

#### D. Generar el Archivo PDF
ZettelTeX tiene su propio pipeline de renderizado que sabe exactamente qué notas necesitan recompilación:
```bash
zetteltex render espacio_metrico
```
*(Si usas bibliografía, puedes indicarle que use biber: `zetteltex render espacio_metrico pdf true`)*.

---

## 📚 Siguiente nivel y Referencias

Una vez que domines estos comandos básicos, puedes ir explorando funcionalidades más avanzadas. La documentación está estructurada funcionalmente:

1. **Uso Avanzado y Flujo Diario**: [Guía de Usuario](01-guia-usuario/flujo-diario.md), para entender cómo usar ZettelTeX en el día a día. Refina tu trabajo explorando [Búsqueda Fuzzy](01-guia-usuario/fuzzy.md) o cómo [Exportar a Markdown](01-guia-usuario/exportacion.md).
2. **Referencia de Comandos**: En caso de duda sobre qué hace un comando en particular, tienes el [Catálogo de comandos interactivos](03-comandos/00-referencia.md).
3. **Detalles internos (Para Colaboradores)**: Si te interesa cómo funciona realmente o deseas contribuir al proyecto en Rust interactuando con SQLite o con los crates nativos ([`zetteltex-parser`, etc.]), visita la [Guía Técnica de la Arquitectura](02-guia-tecnica/README.md).

Ver tambien:

- [Indice maestro](00-indice/README.md)
- [Rutas de lectura por perfil](00-indice/rutas-lectura.md)
