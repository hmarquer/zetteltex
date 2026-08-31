use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};

use anyhow::Result;

use zetteltex_core::WorkspacePaths;

use crate::i18n::tr;
use crate::render::{render_note_cmd, render_project_cmd, render_updates_cmd};
use crate::util::{resolve_note_or_project, TargetKind};

type Snapshot = BTreeMap<PathBuf, SystemTime>;

/// Watch for changes to LaTeX files and recompile the affected notes/projects.
///
/// With a `name` it watches only that note (or project, with `--project`);
/// without one it watches the whole workspace and recompiles everything that
/// went stale, reusing the same staleness logic as `render_updates`.
pub fn watch_cmd(
    paths: &WorkspacePaths,
    name: Option<&str>,
    project: bool,
    format: &str,
    workers: usize,
    poll_ms: u64,
) -> Result<()> {
    match name {
        Some(target) => watch_target(paths, target, project, format, poll_ms),
        None => watch_workspace(paths, format, workers, poll_ms),
    }
}

fn watch_workspace(
    paths: &WorkspacePaths,
    format: &str,
    workers: usize,
    poll_ms: u64,
) -> Result<()> {
    println!(
        "{} ({})",
        tr!("Vigilando el workspace...", "Watching workspace..."),
        tr!("Ctrl-C para detener", "Ctrl-C to stop")
    );
    let mut snapshot = snapshot_workspace(paths)?;
    loop {
        thread::sleep(Duration::from_millis(poll_ms));
        let current = snapshot_workspace(paths)?;
        if current != snapshot {
            println!(
                "{}",
                tr!(
                    "Cambio detectado; recompilando...",
                    "Change detected; recompiling..."
                )
            );
            if let Err(e) = render_updates_cmd(paths, format, workers) {
                eprintln!(
                    "{}: {e}",
                    tr!("Error al recompilar", "Error while recompiling")
                );
            }
            snapshot = current;
        }
    }
}

fn watch_target(
    paths: &WorkspacePaths,
    target: &str,
    project: bool,
    format: &str,
    poll_ms: u64,
) -> Result<()> {
    let kind = resolve_note_or_project(paths, target, project)?;
    let label = match kind {
        TargetKind::Note => tr!("nota", "note"),
        TargetKind::Project => tr!("proyecto", "project"),
    };
    println!(
        "{} '{}' [{}] ({})",
        tr!("Vigilando", "Watching"),
        target,
        label,
        tr!("Ctrl-C para detener", "Ctrl-C to stop")
    );
    // Render once up front, then keep recompiling on changes.
    render_target(paths, target, kind, format)?;
    let mut snapshot = snapshot_target(paths, target, kind)?;
    loop {
        thread::sleep(Duration::from_millis(poll_ms));
        let current = snapshot_target(paths, target, kind)?;
        if current != snapshot {
            if let Err(e) = render_target(paths, target, kind, format) {
                eprintln!(
                    "{}: {e}",
                    tr!("Error al recompilar", "Error while recompiling")
                );
            }
            snapshot = current;
        }
    }
}

fn render_target(paths: &WorkspacePaths, name: &str, kind: TargetKind, format: &str) -> Result<()> {
    println!(
        "{} '{}'",
        tr!(
            "Cambio detectado; recompilando",
            "Change detected; recompiling"
        ),
        name
    );
    match kind {
        TargetKind::Note => render_note_cmd(paths, name, format, false)?,
        TargetKind::Project => render_project_cmd(paths, name, format, false)?,
    }
    Ok(())
}

fn snapshot_workspace(paths: &WorkspacePaths) -> Result<Snapshot> {
    let mut map = Snapshot::new();
    collect_tex(&paths.notes_slipbox, &mut map)?;
    collect_tex(&paths.projects, &mut map)?;
    Ok(map)
}

fn snapshot_target(paths: &WorkspacePaths, target: &str, kind: TargetKind) -> Result<Snapshot> {
    let mut map = Snapshot::new();
    let root = match kind {
        TargetKind::Note => paths.notes_slipbox.join(format!("{target}.tex")),
        TargetKind::Project => paths.projects.join(target),
    };
    if root.is_dir() {
        collect_tex(&root, &mut map)?;
    } else if root.is_file() {
        if let Ok(mtime) = std::fs::metadata(&root).and_then(|m| m.modified()) {
            map.insert(root, mtime);
        }
    }
    Ok(map)
}

fn collect_tex(dir: &Path, map: &mut Snapshot) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_tex(&path, map)?;
        } else if path.extension().map(|e| e == "tex").unwrap_or(false) {
            if let Ok(mtime) = entry.metadata().and_then(|m| m.modified()) {
                map.insert(path, mtime);
            }
        }
    }
    Ok(())
}
