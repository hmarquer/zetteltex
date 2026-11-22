use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::Path;

use anyhow::Result;
use zetteltex_core::WorkspacePaths;
use zetteltex_db::{CitationEntry, Database};
use zetteltex_parser::{parse_bibliography_keys, parse_note, parse_project_inclusions};

use crate::fuzzy::load_zetteltex_config;

/// Indexa las claves del `.bib` configurado (`[bibliography] file`) en la BD.
/// Si el archivo no existe, se indexa una lista vacía: renderizar no depende
/// de que la bibliografía esté presente, pero la BD refleja lo que hay en disco.
pub fn sync_bibliography(db: &Database, paths: &WorkspacePaths) -> Result<()> {
    let bib_path = load_zetteltex_config(paths).bibliography_path(&paths.root);
    let keys = fs::read_to_string(&bib_path)
        .map(|content| parse_bibliography_keys(&content))
        .unwrap_or_default();
    db.replace_bibliography_keys(&keys)?;
    Ok(())
}

/// Las plantillas LaTeX cargan `../../bibliography.bib`. Si el `.bib`
/// configurado tiene otro nombre, se copia a esa ruta para que biber lo
/// encuentre al compilar; así la configuración también rige el render.
pub fn ensure_template_bibliography(paths: &WorkspacePaths) -> Result<()> {
    let configured = load_zetteltex_config(paths).bibliography_path(&paths.root);
    let default_path = paths.root.join("bibliography.bib");
    if configured == default_path || !configured.is_file() {
        return Ok(());
    }
    fs::copy(&configured, &default_path)?;
    Ok(())
}

/// Citas efectivas de una nota: las suyas y las de las notas que transcluye.
pub fn note_citation_entries(paths: &WorkspacePaths, name: &str) -> Result<Vec<CitationEntry>> {
    let mut visited = BTreeSet::new();
    let mut pending = VecDeque::from([name.to_string()]);
    let mut entries = BTreeSet::new();

    while let Some(note_name) = pending.pop_front() {
        if !visited.insert(note_name.clone()) {
            continue;
        }
        let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
        let Ok(content) = fs::read_to_string(&note_path) else {
            continue;
        };
        let source_file = format!("{note_name}.tex");
        add_citations(&content, &source_file, &mut entries, &mut pending);
    }

    Ok(entries
        .into_iter()
        .map(|(key, source)| CitationEntry {
            citation_key: key,
            source_file: source,
        })
        .collect())
}

/// Citas efectivas de un proyecto: las de **todos** sus `.tex` y las de las
/// notas transcluidas desde cualquiera de ellos (transclusiones anidadas
/// incluidas). Es la misma información que se guarda en `project_citation`.
pub fn project_citation_entries(paths: &WorkspacePaths, name: &str) -> Result<Vec<CitationEntry>> {
    let project_dir = paths.projects.join(name);
    let mut tex_files = Vec::new();
    crate::sync::collect_tex_files(&project_dir, &mut tex_files)?;
    tex_files.sort();

    let mut entries = BTreeSet::new();
    let mut pending_notes = VecDeque::new();
    for tex_file in tex_files {
        let Ok(content) = fs::read_to_string(&tex_file) else {
            continue;
        };
        let source_file = relative_source_file(&tex_file, &project_dir);
        add_citations(&content, &source_file, &mut entries, &mut pending_notes);
    }

    let mut visited_notes = BTreeSet::new();
    while let Some(note_name) = pending_notes.pop_front() {
        if !visited_notes.insert(note_name.clone()) {
            continue;
        }
        let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
        let Ok(content) = fs::read_to_string(&note_path) else {
            continue;
        };
        let source_file = format!("notes/slipbox/{note_name}.tex");
        add_citations(&content, &source_file, &mut entries, &mut pending_notes);
    }

    Ok(entries
        .into_iter()
        .map(|(key, source)| CitationEntry {
            citation_key: key,
            source_file: source,
        })
        .collect())
}

fn add_citations(
    content: &str,
    source_file: &str,
    entries: &mut BTreeSet<(String, String)>,
    pending_notes: &mut VecDeque<String>,
) {
    for key in parse_note(content).citations {
        entries.insert((key, source_file.to_string()));
    }
    for inclusion in parse_project_inclusions(content) {
        let note_name = inclusion
            .note_filename
            .trim()
            .trim_end_matches(".tex")
            .to_string();
        if !note_name.is_empty() {
            pending_notes.push_back(note_name);
        }
    }
}

fn relative_source_file(file: &Path, base: &Path) -> String {
    file.strip_prefix(base)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::relative_source_file;
    use std::path::Path;

    #[test]
    fn source_files_use_forward_slashes() {
        assert_eq!(
            relative_source_file(
                Path::new("/ws/projects/p/ch/ch.tex"),
                Path::new("/ws/projects/p")
            ),
            "ch/ch.tex"
        );
    }
}
