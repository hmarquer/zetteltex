**Instalación del binario (GitHub Releases)**

- IMPORTANTE: Antes de crear notas o intentar renderizar, ejecuta `init` en tu workspace para crear el directorio `template/` con las plantillas necesarias. Muchas operaciones (crear notas, proyectos y renderizar con TeX) dependen de los archivos en `workspace/template/` y fallarán si no existen.

- **Descarga**: Ve a la página de Releases del repositorio en GitHub y descarga el artefacto apropiado para tu plataforma (Linux/macOS/Windows).
- **Permisos (Linux/macOS)**: marca el binario como ejecutable:

```bash
chmod +x zetteltex
sudo mv zetteltex /usr/local/bin/
```

- **Windows**: descarga `zetteltex.exe` y colócalo en una carpeta del `PATH`.

Uso básico:

```bash
zetteltex --workspace-root /ruta/al/workspace init
zetteltex --workspace-root /ruta/al/workspace list_recent_files
```

Sobre los archivos en `template/`:

- El proyecto incorpora las plantillas principales en el propio binario en tiempo de compilación (se usan `include_str!(".../template/...")`). Esto significa que el binario ya contiene versiones por defecto de `note.tex`, `project.tex`, `style.sty`, `texbook.cls` y `texnote.cls`.
- El comando `init` crea el directorio `template/` en el workspace y escribe allí las plantillas que vienen empaquetadas si no existen. Por tanto:
  - No hay problema si distribuyes el binario: las plantillas estarán disponibles tras ejecutar `init`.
  - Si quieres personalizar plantillas, edita los ficheros en el `workspace/template/` después de `init`.

Notas sobre empaquetado y mantenimiento:

- Como las plantillas están embebidas, cualquier cambio a las plantillas en el repo requiere recompilar el binario para que la versión distribuida las incluya.
- Si prefieres permitir que el binario use plantillas externas por defecto (por ejemplo, para actualizaciones fuera de releases), habría que cambiar la lógica para buscar plantillas en disco primero y fallar sobre el embebido.

Si quieres, puedo:

- Añadir verificación en runtime que indique si el `template/` fue creado por `init` o es una sobrescritura de usuario.
- Preparar una release de prueba (crear tag y lanzar workflow) y verificar los artefactos en GitHub.
