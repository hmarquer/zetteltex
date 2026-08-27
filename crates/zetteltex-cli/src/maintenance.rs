use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::Result;
use zetteltex_core::WorkspacePaths;
use zetteltex_db::init_database;

use crate::export::{export_notes_dir, export_projects_dir};

pub fn clean_cmd(paths: &WorkspacePaths) -> Result<(usize, usize)> {
    // Remove orphan pdf and markdown files under export directories
    let mut removed_pdf = 0usize;
    let mut removed_md = 0usize;

    let export_notes = export_notes_dir(paths);
    let export_projects = export_projects_dir(paths);
    let legacy_md = paths.root.join("markdown");
    let legacy_pdf = paths.root.join("jabberwocky/adjuntos/pdf");
    let public_pdf = paths.root.join("pdf");

    let mut keep_files = HashSet::new();
    // Build set of valid exported file basenames from DB
    let db = init_database(&paths.root.join("slipbox.db"))?;
    for note in db.list_notes()? {
        keep_files.insert(format!("{}.md", note.filename));
        keep_files.insert(format!("{}.pdf", note.filename));
    }
    for project in db.list_projects()? {
        keep_files.insert(format!("{}.md", project.name));
        keep_files.insert(format!("{}.pdf", project.name));
    }

    fn scan_and_remove(dir: &Path, keep_files: &HashSet<String>) -> Result<(usize, usize)> {
        let mut removed_p = 0usize;
        let mut removed_m = 0usize;
        if !dir.exists() {
            return Ok((0, 0));
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let (rp, rm) = scan_and_remove(&path, keep_files)?;
                removed_p += rp;
                removed_m += rm;
                continue;
            }
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !keep_files.contains(name) {
                    if path.extension().and_then(|e| e.to_str()) == Some("pdf") {
                        fs::remove_file(&path)?;
                        removed_p += 1;
                    } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        fs::remove_file(&path)?;
                        removed_m += 1;
                    }
                }
            }
        }
        Ok((removed_p, removed_m))
    }

    let (rp, rm) = scan_and_remove(&export_notes, &keep_files)?;
    removed_pdf += rp;
    removed_md += rm;
    let (rp, rm) = scan_and_remove(&export_projects, &keep_files)?;
    removed_pdf += rp;
    removed_md += rm;
    let (rp, rm) = scan_and_remove(&legacy_md, &keep_files)?;
    removed_pdf += rp;
    removed_md += rm;
    let (rp, rm) = scan_and_remove(&legacy_pdf, &keep_files)?;
    removed_pdf += rp;
    removed_md += rm;
    let (rp, rm) = scan_and_remove(&public_pdf, &keep_files)?;
    removed_pdf += rp;
    removed_md += rm;

    Ok((removed_pdf, removed_md))
}

pub fn remove_duplicate_citations_cmd(paths: &WorkspacePaths) -> Result<()> {
    use crate::i18n::tr;
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let removed = db.remove_duplicate_citations()?;
    if removed > 0 {
        println!(
            "{}",
            tr!(
                "Eliminada(s) {} cita(s) duplicada(s)",
                "Removed {} duplicate citation(s)",
                removed
            )
        );
    } else {
        println!(
            "{}",
            tr!(
                "No se encontraron citas duplicadas",
                "No duplicate citations found"
            )
        );
    }
    Ok(())
}
