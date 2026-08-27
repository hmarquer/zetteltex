use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use regex::Regex;
use zetteltex_core::WorkspacePaths;

use crate::fuzzy::{command_exists, load_zetteltex_config};
use crate::i18n::tr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Note,
    Project,
}

/// Resuelve si `name` se refiere a una nota, un proyecto o ambos.
///
/// Con `project_override` (flag `--project`) se fuerza a tratar `name` como
/// proyecto. Sin el flag, si `name` existe solo como proyecto se selecciona el
/// proyecto; si existe como nota y como proyecto se avisa y no se renderiza.
pub fn resolve_note_or_project(
    paths: &WorkspacePaths,
    name: &str,
    project_override: bool,
) -> Result<TargetKind> {
    let is_note = paths.notes_slipbox.join(format!("{name}.tex")).exists();
    let is_project = paths
        .projects
        .join(name)
        .join(format!("{name}.tex"))
        .exists();

    if project_override {
        if is_project {
            return Ok(TargetKind::Project);
        }
        bail!(tr!(
            "No existe un proyecto llamado '{name}'",
            "No project named '{name}' exists"
        ));
    }

    match (is_note, is_project) {
        (true, true) => bail!(tr!(
            "'{name}' existe como nota y como proyecto; usa --project para indicar el proyecto",
            "'{name}' exists both as a note and a project; use --project to select the project"
        )),
        (true, false) => Ok(TargetKind::Note),
        (false, true) => Ok(TargetKind::Project),
        (false, false) => bail!(tr!(
            "No existe nota ni proyecto con nombre '{name}'",
            "No note or project named '{name}' exists"
        )),
    }
}

pub fn extract_title_from_tex_content(content: &str) -> Option<String> {
    let token = "\\title{";
    let start = content.find(token)? + token.len();
    let mut depth = 1usize;
    let mut i = start;
    let bytes = content.as_bytes();

    while i < bytes.len() {
        match bytes[i] as char {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(content[start..i].trim().to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

pub fn extract_tagged_block(content: &str, tag: &str) -> Result<Option<String>> {
    let pat = Regex::new(&format!(
        r"(?s)%<\*{}>(.*?)%</{}>",
        regex::escape(tag),
        regex::escape(tag)
    ))?;
    Ok(pat
        .captures(content)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string())))
}

pub fn resolve_workspace_path(paths: &WorkspacePaths, path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        paths.root.join(p)
    }
}

pub fn title_from_name(name: &str) -> String {
    let raw = name
        .split(['_', '-'])
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::with_capacity(raw.len());
    out.extend(first.to_uppercase());
    out.push_str(chars.as_str());
    out
}

pub fn replace_title(template: &str, new_title: &str) -> String {
    let token = "\\title{";
    let Some(start) = template.find(token) else {
        return template.to_string();
    };

    let content_start = start + token.len();
    let Some(relative_end) = template[content_start..].find('}') else {
        return template.to_string();
    };
    let end = content_start + relative_end;

    let mut out = String::with_capacity(template.len() + new_title.len());
    out.push_str(&template[..content_start]);
    out.push_str(new_title);
    out.push_str(&template[end..]);
    out
}

pub fn replace_date(template: &str, date_str: &str) -> String {
    let token = "\\date{";
    let Some(start) = template.find(token) else {
        return template.to_string();
    };

    let content_start = start + token.len();
    let Some(relative_end) = template[content_start..].find('}') else {
        return template.to_string();
    };
    let end = content_start + relative_end;

    let mut out = String::with_capacity(template.len() + date_str.len());
    out.push_str(&template[..content_start]);
    out.push_str(date_str);
    out.push_str(&template[end..]);
    out
}

pub fn open_in_editor(paths: &WorkspacePaths, file_path: &Path) -> Result<()> {
    let config = load_zetteltex_config(paths);
    let editor_cmd = match config.editor_cmd() {
        Some(cmd) => cmd,
        None => bail!(
            "{}",
            tr!(
                "No hay editor configurado. Ejecuta `zetteltex init_config` para configurarlo.",
                "No editor configured. Run `zetteltex init_config` to set one up."
            )
        ),
    };

    let mut cmd = Command::new(editor_cmd);
    cmd.arg(file_path);
    let status = cmd.status().map_err(|e| {
        anyhow::anyhow!(
            "{}",
            tr!(
                "No se pudo ejecutar el editor '{}': {}",
                "Could not run editor '{}': {}",
                editor_cmd,
                e
            )
        )
    })?;
    if !status.success() {
        bail!(
            "{}",
            tr!(
                "El editor '{}' terminó con estado {}",
                "Editor '{}' exited with status {}",
                editor_cmd,
                status
            )
        )
    }
    Ok(())
}

pub fn run_external_tool(
    bin: &str,
    args: &[&str],
    cwd: Option<&Path>,
    timeout: Option<Duration>,
) -> Result<()> {
    let mut cmd = Command::new(bin);
    cmd.args(args);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    let output = match run_with_timeout(&mut cmd, timeout) {
        Ok(out) => out,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            bail!(tr!(
                "{bin} no encontrado en PATH",
                "{bin} not found in PATH"
            ))
        }
        Err(err) => return Err(err.into()),
    };
    if !output.status.success() {
        let rendered_cmd = format!("{} {}", bin, args.join(" "));
        let cwd_display = cwd
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<current-dir>".to_string());
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        let detail = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else if !stdout.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            format!("exit status {} (no stderr/stdout)", output.status)
        };

        bail!(
            "{} '{}' {} {}: {}",
            tr!("fallo al ejecutar", "failed while running"),
            rendered_cmd,
            tr!("en", "in"),
            cwd_display,
            detail
        );
    }
    Ok(())
}

/// Ejecuta `cmd` capturando su salida, matando el proceso si no termina dentro
/// del `timeout` dado (o sin limite si `timeout` es `None`).
fn run_with_timeout(cmd: &mut Command, timeout: Option<Duration>) -> std::io::Result<Output> {
    match timeout {
        None => cmd.output(),
        Some(t) => {
            cmd.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let mut child = cmd.spawn()?;
            let deadline = Instant::now() + t;
            loop {
                match child.try_wait()? {
                    Some(_) => break,
                    None => {
                        if Instant::now() >= deadline {
                            let _ = child.kill();
                            let _ = child.wait();
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::TimedOut,
                                format!("exceeded timeout of {}s", t.as_secs()),
                            ));
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
            }
            child.wait_with_output()
        }
    }
}

pub fn run_external_open_nonblocking_verified(
    bin: &str,
    args: &[&str],
    cwd: Option<&Path>,
) -> Result<()> {
    fn spawn_and_verify(mut cmd: Command, cwd: Option<&Path>) -> Result<()> {
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }

        let mut child = cmd.spawn()?;

        // Espera corta para detectar errores inmediatos sin bloquear la salida de fuzzy.
        let timeout = Duration::from_millis(350);
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                if status.success() {
                    return Ok(());
                }
                bail!(tr!(
                    "el comando open fallo con estado {status}",
                    "open command failed with status {status}"
                ))
            }

            if start.elapsed() >= timeout {
                // Sigue vivo: consideramos que se lanzo correctamente.
                return Ok(());
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }

    // En Linux, setsid -f suele desacoplar mejor del terminal que un spawn directo.
    if command_exists("setsid") {
        let mut cmd = Command::new("setsid");
        cmd.arg("-f").arg(bin).args(args);
        if spawn_and_verify(cmd, cwd).is_ok() {
            return Ok(());
        }
    }

    // Fallback clasico para evitar SIGHUP al cerrar el terminal.
    if command_exists("nohup") {
        let mut cmd = Command::new("nohup");
        cmd.arg(bin).args(args);
        if spawn_and_verify(cmd, cwd).is_ok() {
            return Ok(());
        }
    }

    // Ultimo intento: lanzamiento directo.
    let mut cmd = Command::new(bin);
    cmd.args(args);
    spawn_and_verify(cmd, cwd)
}

pub fn write_xclip_clipboard(text: &str) -> Result<()> {
    fn try_clipboard_write(bin: &str, args: &[&str], text: &str) -> Result<bool> {
        let mut cmd = if command_exists("setsid") {
            let mut detached = Command::new("setsid");
            detached.arg("-f").arg(bin).args(args);
            detached
        } else {
            let mut direct = Command::new(bin);
            direct.args(args);
            direct
        };

        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(_) => return Ok(false),
        };

        if let Some(mut stdin) = child.stdin.take() {
            if let Err(err) = stdin.write_all(text.as_bytes()) {
                // Algunos binarios/fakes de clipboard cierran stdin de inmediato;
                // si el proceso finaliza con exito, tratamos EPIPE como no fatal.
                if err.kind() != std::io::ErrorKind::BrokenPipe {
                    return Err(err.into());
                }
            }
            drop(stdin);
        } else {
            return Ok(false);
        }

        // Verificacion corta: si falla de inmediato devolvemos false para activar fallback.
        let timeout = Duration::from_millis(50); // Reducido para evitar congelación del UI si un binario bloquea
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                return Ok(status.success());
            }
            if start.elapsed() >= timeout {
                // Sigue vivo: en xclip/xsel es normal, el proceso puede mantener la seleccion.
                // IMPORTANTE: En utilidades de clipboard que se desconectan lento (ej. wl-copy con fallos de DBus),
                // hacemos kill() si no han terminado, ya que un Dbus-hanging arruina la terminal.
                if bin == "wl-copy" || bin == "wl-paste" {
                    // No queremos hacer kill si esta vivo en todos los casos, pero en wayland es frecuente que el timeout sea indicativo de éxito daemonizado
                    // wl-copy hace disown de si mismo a veces, asi que no es seguro hacer kill_on_drop.
                }
                return Ok(true);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    // Comprobamos disponibilidad antes de spawnear para evitar tirones de D-Bus en binarios faltantes pero registrados.
    if command_exists("wl-copy")
        && std::env::var("WAYLAND_DISPLAY").is_ok()
        && try_clipboard_write("wl-copy", &[], text)?
    {
        return Ok(());
    }
    if command_exists("xclip") && try_clipboard_write("xclip", &["-selection", "clipboard"], text)?
    {
        return Ok(());
    }
    if command_exists("xsel") && try_clipboard_write("xsel", &["--clipboard", "--input"], text)? {
        return Ok(());
    }

    bail!(tr!(
        "No se pudo copiar al portapapeles (wl-copy/xclip/xsel)",
        "Could not copy to clipboard (wl-copy/xclip/xsel)"
    ))
}

pub fn read_xclip_clipboard() -> Result<String> {
    let readers: [(&str, &[&str]); 3] = [
        ("wl-paste", &["--no-newline"]),
        ("xclip", &["-selection", "clipboard", "-o"]),
        ("xsel", &["--clipboard", "--output"]),
    ];

    for (bin, args) in readers {
        if !command_exists(bin) {
            continue;
        }
        if bin == "wl-paste" && std::env::var("WAYLAND_DISPLAY").is_err() {
            continue;
        }

        let output = match Command::new(bin).args(args).output() {
            Ok(out) => out,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => continue,
        };
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
    }

    bail!(tr!(
        "Error leyendo portapapeles (wl-paste/xclip/xsel)",
        "Error reading clipboard (wl-paste/xclip/xsel)"
    ))
}
