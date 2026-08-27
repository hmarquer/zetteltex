use std::path::PathBuf;
use std::{fs, process::ExitCode};

use anyhow::{bail, Result};
use chrono::Utc;
use zetteltex_core::WorkspacePaths;
use zetteltex_db::init_database;
use zetteltex_parser::parse_note;

use crate::i18n::tr;
use crate::sync::{note_stem_from_path, synchronize_notes, synchronize_projects};
use crate::util::{open_in_editor, replace_date, replace_title, title_from_name};
use crate::workspace::read_template_file_or_suggest_init;

pub fn create_project(paths: &WorkspacePaths, project_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if db.project_id_by_name(project_name)?.is_some() {
        bail!(tr!(
            "Ya existe un proyecto con nombre {project_name} en la base de datos",
            "A project with name {project_name} already exists in the database"
        ));
    }

    let project_dir = paths.projects.join(project_name);
    fs::create_dir_all(&project_dir)?;

    let project_filename = format!("{project_name}.tex");
    let project_tex_path = project_dir.join(&project_filename);
    if !project_tex_path.exists() {
        let template = read_template_file_or_suggest_init(paths, "project.tex")?;
        let title = title_from_name(project_name);
        let date = Utc::now().format("%d-%m-%Y").to_string();
        let updated = replace_date(&replace_title(&template, &title), &date);
        fs::write(&project_tex_path, updated)?;
    }

    db.upsert_project(project_name, &project_filename, Utc::now())?;
    println!(
        "{}",
        tr!(
            "Proyecto {} creado en {}",
            "Project {} created at {}",
            project_name,
            project_dir.display()
        )
    );
    Ok(())
}

pub fn create_note(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if db.note_exists(note_name)? {
        bail!(
            "{}",
            tr!("Ya existe una nota con nombre {note_name} en la base de datos. Si no es el caso, ejecuta `zetteltex synchronize` e intentalo de nuevo", "A note with file name {note_name} already exists in the database. If this is not the case then run zetteltex synchronize and try again")
        );
    }

    let note_tex_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    if !note_tex_path.exists() {
        let template = read_template_file_or_suggest_init(paths, "note.tex")?;
        let title = title_from_name(note_name);
        let date = Utc::now().format("%d-%m-%Y").to_string();
        let updated = replace_date(&replace_title(&template, &title), &date);
        fs::write(&note_tex_path, updated)?;
    } else {
        println!(
            "{}",
            tr!(
                "El archivo {} ya existe, se omite copiar la plantilla",
                "File {} already exists, skipping copying the template",
                note_tex_path.display()
            )
        );
    }

    add_to_documents(paths, note_name)?;

    let default_title = title_from_name(note_name);
    db.upsert_note(note_name, &default_title, Utc::now())?;
    Ok(())
}

pub fn list_recent_files(paths: &WorkspacePaths, n: usize) -> Result<()> {
    let recent = recent_note_names(paths)?;
    if recent.is_empty() {
        println!(
            "{}",
            tr(
                "No se encontraron notas en la base de datos.",
                "No notes found in database."
            )
        );
        return Ok(());
    }

    for (idx, name) in recent.into_iter().take(n).enumerate() {
        println!("{}:\t{}", idx + 1, name);
    }

    Ok(())
}

pub fn list_unreferenced(paths: &WorkspacePaths) -> Result<()> {
    let _ = synchronize_notes(paths)?;
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let notes = db.list_unreferenced_notes()?;

    if notes.is_empty() {
        println!(
            "{}",
            tr(
                "No se encontraron notas sin referenciar.",
                "No unreferenced notes found."
            )
        );
        return Ok(());
    }

    for (idx, note) in notes.iter().enumerate() {
        println!("{}: {}", idx + 1, note);
    }

    Ok(())
}

pub fn add_to_documents(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let documents_path = paths.root.join("notes").join("documents.tex");
    let line = format!("\\externaldocument[{note_name}-]{{{note_name}}}\n");
    let mut current = String::new();
    if documents_path.exists() {
        current = fs::read_to_string(&documents_path)?;
    }
    if !current.contains(&line) {
        current.push_str(&line);
        fs::write(&documents_path, current)?;
    }

    Ok(())
}

pub fn recent_note_names(paths: &WorkspacePaths) -> Result<Vec<String>> {
    let mut entries: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        if note_stem_from_path(&path).is_none() {
            continue;
        }
        let modified = fs::metadata(&path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        entries.push((modified, path));
    }

    entries.sort_by(|a, b| b.0.cmp(&a.0));

    let names = entries
        .into_iter()
        .filter_map(|(_, path)| note_stem_from_path(&path))
        .collect::<Vec<_>>();

    Ok(names)
}

pub fn list_citations(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if !db.note_exists(note_name)? {
        bail!(tr("Consulta sin resultados", "Query returned no rows"));
    }

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    let content = fs::read_to_string(&note_path)?;
    let parsed = parse_note(&content)?;

    let mut unique = std::collections::BTreeSet::new();
    for citation in parsed.citations {
        unique.insert(citation);
    }

    for citation in unique {
        println!("{citation}");
    }

    Ok(())
}

pub fn edit_cmd(paths: &WorkspacePaths, filename: Option<&str>) -> Result<()> {
    let note_name = match filename {
        Some(name) if !name.trim().is_empty() => name.to_string(),
        _ => {
            let recent = recent_note_names(paths)?;
            recent.into_iter().next().ok_or_else(|| {
                anyhow::anyhow!(tr("No hay notas para editar", "No notes found to edit"))
            })?
        }
    };

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    if !note_path.exists() {
        bail!(
            "{}: {}",
            tr("El archivo no existe", "No such file"),
            note_path.display()
        );
    }

    open_in_editor(paths, &note_path)?;
    Ok(())
}

pub fn list_projects_cmd(paths: &WorkspacePaths) -> Result<ExitCode> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let projects = db.list_projects()?;
    if projects.is_empty() {
        println!(
            "{}",
            tr(
                "No se encontraron proyectos en la base de datos.",
                "No projects found in database."
            )
        );
        return Ok(ExitCode::SUCCESS);
    }

    println!("{}", tr("Proyectos:", "Projects:"));
    for (idx, project) in projects.iter().enumerate() {
        println!("{}:\t{}", idx + 1, project.name);
    }
    Ok(ExitCode::SUCCESS)
}

pub fn list_project_inclusions_cmd(paths: &WorkspacePaths, project: &str) -> Result<ExitCode> {
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let inclusions = db.list_project_inclusions_by_name(project)?;

    if inclusions.is_empty() {
        println!(
            "{} {}",
            tr(
                "No se encontraron inclusiones para el proyecto",
                "No inclusions found for project"
            ),
            project
        );
        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "{} \"{project}\":",
        tr("Inclusiones en el proyecto", "Inclusions in project")
    );
    for (idx, inc) in inclusions.iter().enumerate() {
        if inc.tag.is_empty() {
            println!(
                "{}. {} {} {}",
                idx + 1,
                inc.note_filename,
                tr("(en)", "(in)"),
                inc.source_file
            );
        } else {
            println!(
                "{}. {} [tag: {}] {} {}",
                idx + 1,
                inc.note_filename,
                inc.tag,
                tr("(en)", "(in)"),
                inc.source_file
            );
        }
    }
    println!(
        "{}: {} {}",
        tr("Total", "Total"),
        inclusions.len(),
        tr("notas incluidas", "notes included")
    );
    Ok(ExitCode::SUCCESS)
}

pub fn list_note_projects_cmd(paths: &WorkspacePaths, note: &str) -> Result<ExitCode> {
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let projects = db.list_note_projects(note)?;

    if projects.is_empty() {
        println!(
            "{}",
            tr!(
                "La nota {note} no esta incluida en ningun proyecto",
                "Note {note} is not included in any project"
            )
        );
        return Ok(ExitCode::SUCCESS);
    }

    println!(
        "{} \"{note}\":",
        tr("Proyectos que incluyen la nota", "Projects including note")
    );
    for (idx, p) in projects.iter().enumerate() {
        if p.tag.is_empty() {
            println!("{}. {}/{}", idx + 1, p.project_name, p.source_file);
        } else {
            println!(
                "{}. {}/{} [tag: {}]",
                idx + 1,
                p.project_name,
                p.source_file,
                p.tag
            );
        }
    }
    println!(
        "{}: {} {}",
        tr("Total", "Total"),
        projects.len(),
        tr("proyectos", "projects")
    );
    Ok(ExitCode::SUCCESS)
}
