use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs};

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use zetteltex_core::WorkspacePaths;
use zetteltex_db::Database;
use zetteltex_parser::{parse_note, parse_project_inclusions, Reference};
use crate::util::extract_title_from_tex_content;

const RENDER_TEMP_PREFIX: &str = ".zetteltex-render-";

struct TransactionGuard<'a> {
    db: &'a Database,
    committed: bool,
}

impl<'a> TransactionGuard<'a> {
    fn new(db: &'a Database) -> Self {
        Self {
            db,
            committed: false,
        }
    }

    fn commit(mut self) -> Result<()> {
        self.db.commit_transaction()?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for TransactionGuard<'_> {
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.db.rollback_transaction();
        }
    }
}

#[derive(Debug)]
pub struct SyncStats {
    pub notes_synced: usize,
    pub links_built: usize,
    pub unresolved_references: usize,
}

#[derive(Debug)]
pub struct ProjectSyncStats {
    pub projects_synced: usize,
    pub inclusions_synced: usize,
    pub missing_notes: usize,
}

#[derive(Debug)]
pub struct ValidationIssue {
    pub kind: &'static str,
    pub source: String,
    pub target_note: String,
    pub target_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationScope {
    Notes,
    Projects,
    Both,
}

pub fn synchronize_notes(paths: &WorkspacePaths) -> Result<SyncStats> {
    let db_path = paths.root.join("slipbox.db");
    let db = Database::open(&db_path)?;
    let tx = TransactionGuard::new(&db);

    db.begin_transaction()?;
    let _ = db.delete_notes_with_prefix(RENDER_TEMP_PREFIX)?;

    let mut parsed_by_note = HashMap::new();
    let mut notes_synced = 0usize;

    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        let Some(filename) = note_stem_from_path(&path) else {
            continue;
        };

        let content = fs::read_to_string(&path)?;
        let parsed = parse_note(&content)?;

        let modified = fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::now());
        let modified_utc: DateTime<Utc> = modified.into();

        let title = extract_title_from_tex_content(&content).unwrap_or_else(|| filename.clone());
        let note_id = db.upsert_note(&filename, &title, modified_utc)?;
        db.replace_labels(note_id, &parsed.labels)?;
        db.replace_citations(note_id, &parsed.citations)?;

        parsed_by_note.insert(filename, parsed);
        notes_synced += 1;
    }

    db.clear_links()?;
    let mut links_built = 0usize;
    let mut unresolved_references = 0usize;

    for (source_note, parsed) in parsed_by_note {
        let Some(source_id) = db.note_id_by_filename(&source_note)? else {
            continue;
        };

        for reference in parsed.references {
            if let Some(target_label_id) =
                db.target_label_id(&reference.target_note, &reference.target_label)?
            {
                db.insert_link(source_id, target_label_id)?;
                links_built += 1;
            } else {
                unresolved_references += 1;
            }
        }
    }
    
    tx.commit()?;

    Ok(SyncStats {
        notes_synced,
        links_built,
        unresolved_references,
    })
}

pub fn check_reference(db: &Database, issues: &mut Vec<ValidationIssue>, source: &str, reference: &Reference) -> Result<()> {
    if !db.note_exists(&reference.target_note)? {
        issues.push(ValidationIssue {
            kind: "missing_note",
            source: source.to_string(),
            target_note: reference.target_note.clone(),
            target_label: reference.target_label.clone(),
        });
        return Ok(());
    }

    if !db.label_exists(&reference.target_note, &reference.target_label)? {
        issues.push(ValidationIssue {
            kind: "missing_label",
            source: source.to_string(),
            target_note: reference.target_note.clone(),
            target_label: reference.target_label.clone(),
        });
    }
    Ok(())
}

pub fn validate_references(paths: &WorkspacePaths, scope: ValidationScope) -> Result<Vec<ValidationIssue>> {
    let db = Database::open(&paths.root.join("slipbox.db"))?;
    let mut issues = Vec::new();

    // --- Validate notes in slipbox ---
    if scope == ValidationScope::Notes || scope == ValidationScope::Both {
        for entry in fs::read_dir(&paths.notes_slipbox)? {
            let entry = entry?;
            let path = entry.path();
            let Some(source) = note_stem_from_path(&path) else {
                continue;
            };

            let content = fs::read_to_string(&path)?;
            let parsed = parse_note(&content)?;

            for reference in parsed.references {
                check_reference(&db, &mut issues, &format!("{source}.tex"), &reference)?;
            }

            // \ref{label} in notes: internal to the same file
            for ref_text in &parsed.plain_refs {
                if !parsed.labels.contains(ref_text) {
                    issues.push(ValidationIssue {
                        kind: "missing_label",
                        source: format!("{source}.tex"),
                        target_note: source.clone(),
                        target_label: ref_text.clone(),
                    });
                }
            }
        }
    }

    // --- Validate project files ---
    if (scope == ValidationScope::Projects || scope == ValidationScope::Both) && paths.projects.exists() {
        for entry in fs::read_dir(&paths.projects)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(project_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };

            let mut tex_files = Vec::new();
            collect_tex_files(&path, &mut tex_files)?;

            // First pass: collect all labels across the project
            let mut project_labels: Vec<String> = Vec::new();
            let mut file_entries: Vec<(PathBuf, zetteltex_parser::ParsedNote)> = Vec::new();
            for tex_path in &tex_files {
                let content = fs::read_to_string(tex_path)?;
                let parsed = parse_note(&content)?;
                project_labels.extend(parsed.labels.clone());
                file_entries.push((tex_path.clone(), parsed));
            }

            // Second pass: validate each file
            for (tex_path, parsed) in &file_entries {
                let fname = tex_path.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("?");
                let source_label = format!("projects/{project_name}/{fname}");

                // Validate \transclude
                let content = fs::read_to_string(tex_path)?;
                let inclusions = parse_project_inclusions(&content)?;
                for inc in inclusions {
                    if !db.note_exists(&inc.note_filename)? {
                        issues.push(ValidationIssue {
                            kind: "missing_note",
                            source: source_label.clone(),
                            target_note: inc.note_filename,
                            target_label: String::from("transclude"),
                        });
                    }
                }

                // Validate \excref, \exhyperref, \exref
                for reference in &parsed.references {
                    check_reference(&db, &mut issues, &source_label, reference)?;
                }

                // \ref{label} in projects: resolved against all labels in the project
                for ref_text in &parsed.plain_refs {
                    if !project_labels.contains(ref_text) {
                        issues.push(ValidationIssue {
                            kind: "missing_label",
                            source: source_label.clone(),
                            target_note: format!("projects/{project_name}"),
                            target_label: ref_text.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(issues)
}

pub fn synchronize_projects(paths: &WorkspacePaths) -> Result<ProjectSyncStats> {
    let db_path = paths.root.join("slipbox.db");
    let db = Database::open(&db_path)?;
    let tx = TransactionGuard::new(&db);

    let mut projects_synced = 0usize;
    let mut inclusions_synced = 0usize;
    let missing_notes = 0usize;

    db.begin_transaction()?;

    for entry in fs::read_dir(&paths.projects)? {
        let entry = entry?;
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }

        let Some(project_name) = project_dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let project_filename = format!("{project_name}.tex");
        let project_main = project_dir.join(&project_filename);
        if !project_main.exists() {
            continue;
        }

        let modified = fs::metadata(&project_main)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::now());
        let modified_utc: DateTime<Utc> = modified.into();
        let project_id = db.upsert_project(project_name, &project_filename, modified_utc)?;
        projects_synced += 1;

        let mut tex_files = Vec::new();
        collect_tex_files(&project_dir, &mut tex_files)?;

        let mut resolved_inclusions = Vec::new();
        for tex_file in tex_files {
            let content = fs::read_to_string(&tex_file)?;
            let inclusions = parse_project_inclusions(&content)?;
            let source_file = tex_file
                .strip_prefix(&project_dir)
                .unwrap_or(&tex_file)
                .to_string_lossy()
                .replace('\\', "/");

            for inclusion in inclusions {
                let note_id = resolve_note_id(&db, &inclusion.note_filename)?;
                resolved_inclusions.push((note_id, source_file.clone(), inclusion.tag));
                inclusions_synced += 1;
            }
        }

        db.replace_project_inclusions(project_id, &resolved_inclusions)?;
    }

    tx.commit()?;

    Ok(ProjectSyncStats {
        projects_synced,
        inclusions_synced,
        missing_notes,
    })
}

pub fn collect_tex_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_tex_files(&path, out)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("tex") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn resolve_note_id(db: &zetteltex_db::Database, note_ref: &str) -> Result<i64> {
    let normalized = note_ref.trim().trim_end_matches(".tex");
    match db.note_id_by_filename(normalized)? {
        Some(id) => Ok(id),
        None => bail!(
            "Missing note reference '{note_ref}': transclude must match an existing note filename exactly"
        ),
    }
}

pub fn is_render_temp_note_name(name: &str) -> bool {
    name.starts_with(RENDER_TEMP_PREFIX)
}

pub fn note_stem_from_path(path: &Path) -> Option<String> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("tex") {
        return None;
    }
    let stem = path.file_stem().and_then(|stem| stem.to_str())?;
    if is_render_temp_note_name(stem) {
        return None;
    }
    Some(stem.to_string())
}

#[cfg(test)]
mod tests {
    use super::resolve_note_id;
    use chrono::Utc;
    use tempfile::TempDir;
    use zetteltex_db::Database;

    fn open_db_with_notes(note_names: &[&str]) -> Database {
        let temp = TempDir::new().expect("tempdir");
        let db_path = temp.path().join("slipbox.db");
        let db = Database::open(&db_path).expect("open db");

        for name in note_names {
            db.upsert_note(name, name, Utc::now()).expect("insert note");
        }

        std::mem::forget(temp);
        db
    }

    #[test]
    fn resolve_note_id_matches_exact_filename() {
        let db = open_db_with_notes(&["MyNote", "mynote"]);

        let resolved = resolve_note_id(&db, "MyNote").expect("resolve exact");

        assert_eq!(db.note_id_by_filename("MyNote").unwrap(), Some(resolved));
    }

    #[test]
    fn resolve_note_id_rejects_missing_exact_match() {
        let db = open_db_with_notes(&["mynote", "my-note"]);

        let err = resolve_note_id(&db, "MyNote").expect_err("missing exact reference should fail");

        assert!(err.to_string().contains("Missing note reference"));
    }
}

