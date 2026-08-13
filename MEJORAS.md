# Posibles mejoras del CLI de zetteltex

Este documento recoge posibles mejoras sobre la interfaz CLI de zetteltex,
analizadas sobre el código en `crates/zetteltex-cli`. Cada mejora se resuelve
una a una; su estado se indica en la cabecera.

## 1. Argumentos posicionales fragiles (HECHA)

`render <name> [format] [biber]`, `render_updates [format] [--workers]`, etc.
usan argumentos posicionales para `format` y `biber`.

- `format` y `biber` deberian ser flags (`--format <pdf|html>`, `--biber`) con
  `clap::ValueEnum`, lo que valida el formato en tiempo de parseo (hoy
  "Unsupported format" es un error en runtime con exit code 1), permite
  autocompletado y es extensible a futuros formatos.
- `biber` como posicional `Option<bool>` obliga a escribir literalmente
  `true`/`false`: `render foo pdf true`.
- Inconsistencia interna: `format` es posicional pero `--workers` ya es flag.

Archivos: `crates/zetteltex-cli/src/cli.rs`, `src/main.rs`, docs en
`docs/03-comandos/render/`.

## 2. Proliferacion de comandos redundantes (HECHA)

- `render_all_pdf` es identico a `render_all pdf` (formato hardcodeado).
- Pares duplicados nota/proyecto: `render`/`render_project`, `biber`/
  `biber_project`, `export_markdown`/`export_project_markdown`. Un flag
  `--project` o un grupo de subcomandos reduciria la superficie a la mitad.
- `export_all_markdown` duplica `export_all_notes_markdown` +
  `export_all_projects_markdown`.
- `to_md` vs `export_markdown`: dos caminos de exportacion markdown distintos
  (`markdown/` en la raiz vs vault Obsidian) con nombres confusos.

Resolucion (HECHA):

- Se eliminaron `render_project`, `biber_project`, `render_all_pdf`,
  `export_project_markdown`, `export_all_notes_markdown`,
  `export_all_projects_markdown` y `to_md`.
- `render`, `biber` y `export_markdown` aceptan ahora un flag `--project` y
  detectan automaticamente nota vs proyecto por nombre (avisando si hay
  ambiguedad).
- `export_all_markdown` exporta notas y proyectos por defecto, con flags
  `--notes` y `--projects` para acotar el alcance.

## 3. Salida "Plan render..." enganosa o duplicada (HECHA)

- El plan se computa aparte de la logica real de pasadas, que esta duplicada en
  `render_note_pdf`, `render_project_pdf`, etc.; riesgo de divergencia.
- `render_updates` (html) imprime `biber=true` hardcodeado para proyectos
  (`render.rs`), mientras que notas usa deteccion por nota.
- Para `html`, el plan dice `motor=pdflatex`, que es falso.
- `render_updates` imprime dos resumenes redundantes ("Plan..." y "Render
  updates: N nota(s)...").
- Varios `println!` tienen espacios colgantes (ej. `salida={} `).

Resolucion (HECHA):

- `render_motor` y `render_pass_count` son la fuente unica de verdad del plan
  (motor y pasadas); todos los planes los usan. Para `html` el motor se
  reporta como `make4ht`.
- El plan de `render_updates` se imprime una sola vez, tras sincronizar y con
  los conteos reales (notas/proyectos) y el numero de notas con biber;
  eliminada la linea redundante "Render updates: N nota(s), N proyecto(s)".
- `render_all_projects` (html) reporta `con_biber=<total>` en vez del
  hardcodeado `biber=true`.
- Eliminados los espacios colgantes de los `println!` de planes.

## 4. Errores paralelos tragados (HECHA)

En `run_parallel_render_with_progress` (`render.rs`) solo se conserva el primer
error (`errors.remove(0)`); el resto de fallos se pierde silenciosamente. Con
muchas notas quieres ver todos. Mejora: resumen completo, fichero de log o flag
`--verbose`.

Resolucion (HECHA):

- Tras la fase paralela se imprimen **todos** los errores, uno por linea y
  ordenados por archivo (stderr, con prefijo `  -`), precedidos del resumen
  `{fase} | errores: N`.
- El `bail!` final conserva el total sin descartar detalles (cada error ya se
  listo en completo).

## 5. Sin progreso en modo no-TTY (HECHA)

Cuando stdout no es terminal no se imprime nada por item; solo "completado" al
final. Podria imprimirse `2/500` al completar cada uno.

Resolucion (HECHA):

- En modo no-TTY, cada item completado imprime una linea
  `{fase}: {completados}/{total}` (p. ej. `Render notas: 2/3`), tanto para
  exitos como para fallos, seguida del resumen final ya existente.

## 6. Consistencia de idioma y nombres (HECHA)

- Mensajes mezclan espanol e ingles ("Sincronizacion completa", "Clean
  summary", "Force synchronize", "Plan render").
- Nombres de comandos inconsistentes: `newnote`, `newproject`, `addtodocuments`
  frente al resto en snake_case (`rename_note`, `list_recent_files`,
  `init_config`).

Resolucion (HECHA):

- Se anadio i18n (es/en) con `[general] lang` en `zetteltex.toml`
  (`crates/zetteltex-core/src/i18n.rs`): todos los mensajes de usuario se
  muestran en espanol (default) o ingles segun config, fijado al arranque con
  `set_lang`. La ayuda de clap queda en espanol (limite del derive estatico).
- Los nombres de comando se conservan tal cual; el CLI sigue exponiendo
  `newnote`, `newproject`, `addtodocuments` por compatibilidad.

## 7. Metadatos clap ausentes (HECHA)

Ningun subcomando tiene `about`/`long_about`, asi que `zetteltex render --help`
es casi vacio.

Resolucion (HECHA):

- Todos los subcomandos de `crates/zetteltex-cli/src/cli.rs` tienen ahora
  `about` (descripcion breve) y cada argumento tiene `help`, en espanol. Con
  esto, `--help` (indice) y `<subcomando> --help` son auto-documentados:
  `zetteltex render --help` muestra proposito, argumentos y flags.

## 8. Docs que documentan comandos inexistentes (HECHA)

- `docs/03-comandos/notas/rename_file.md` y `rename_label.md` describen
  comandos (`rename_file <old> <new>`, `rename_label <note> <old> <new>`) que
  no existen en el CLI; este expone `rename_note <name>`, un comando
  interactivo que renombra el archivo y las labels. Corregir: fusionar en un
  solo `rename_note.md` (o eliminar los dos y documentar el real).
- Enlaces rotos preexistentes: faltan `docs/00-indice/README.md`,
  `docs/01-guia-usuario/README.md` y `inicio-rapido.md` (referenciado desde
  `flujo-diario.md`).
- Comandos sin documentar en el catalogo: `init`, `init_config`, `clean`.

Resolucion (HECHA):

- Eliminados `rename_file.md` y `rename_label.md`; creado
  `docs/03-comandos/notas/rename_note.md` con el comando real (interactivo:
  archivo + etiquetas). Actualizados enlaces en catalogo, referencia resumida,
  `rename_recent.md` y la doc de la funcion interna `rename_file`
  (05-funciones-codigo), que sigue documentando la funcion existente.
- Creados los destinos faltantes: `docs/00-indice/README.md`,
  `docs/01-guia-usuario/README.md` y `docs/01-guia-usuario/inicio-rapido.md`.
- Documentados `init`, `init_config` y `clean` en el catalogo
  (`docs/03-comandos/init.md`, `init_config.md`, `utilidades/clean.md`) y en la
  referencia resumida.
- Verificacion automatica de enlaces markdown relativos: sin enlaces rotos.
