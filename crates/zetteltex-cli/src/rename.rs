use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{bail, Result};
use regex::Regex;
use zetteltex_core::WorkspacePaths;
use zetteltex_db::init_database;
use zetteltex_parser::parse_note;

use crate::export::export_notes_dir;
use crate::i18n::tr;
use crate::notes::recent_note_names;
use crate::sync::{collect_tex_files, note_stem_from_path, synchronize_notes};

pub fn rename_recent(paths: &WorkspacePaths, n: usize) -> Result<()> {
    if n == 0 {
        bail!(tr("n debe ser >= 1", "n must be >= 1"));
    }

    let _ = synchronize_notes(paths)?;
    let recent = recent_note_names(paths)?;
    if n > recent.len() {
        bail!(
            "{}",
            tr!(
                "El indice reciente solicitado {n} esta fuera de rango ({} nota(s))",
                "Requested recent index {n} out of range ({} note(s))",
                recent.len()
            )
        );
    }

    let current = recent[n - 1].clone();
    print!(
        "{} [{}]: ",
        tr("Cambiar nombre de archivo a", "Change file name to"),
        current
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let new_name = input.trim();

    if new_name.is_empty() || new_name == current {
        println!("{}", tr("No se realizaron cambios", "No changes made"));
        return Ok(());
    }

    rename_file(paths, &current, new_name)
}

pub fn rename_note(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let _ = synchronize_notes(paths)?;
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if !db.note_exists(note_name)? {
        bail!(tr!(
            "Nota {note_name} no encontrada en la base de datos",
            "Note {note_name} not found in database"
        ));
    }

    // 1. File rename
    print!(
        "{} '{note_name}' [{}]: ",
        tr!("Renombrar archivo", "Rename file"),
        tr!("deja vacio para omitir", "leave empty to skip")
    );
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let new_name = input.trim().to_string();
    let file_renamed = if !new_name.is_empty() && new_name != note_name {
        rename_file(paths, note_name, &new_name)?;
        true
    } else {
        false
    };

    // 2. Label renames — use the (possibly new) filename
    let effective_name = if file_renamed { &new_name } else { note_name };
    let labels = db.labels_for_note(effective_name)?;
    let mut labels_renamed = false;

    for label in &labels {
        print!(
            "{} '{label}' {} '{effective_name}' [{}]: ",
            tr!("Renombrar etiqueta", "Rename label"),
            tr!("en", "in"),
            tr!("deja vacio para omitir", "leave empty to skip")
        );
        io::stdout().flush()?;
        let mut label_input = String::new();
        io::stdin().read_line(&mut label_input)?;
        let new_label = label_input.trim().to_string();
        if new_label.is_empty() || new_label == *label {
            continue;
        }
        rename_label(paths, effective_name, label, &new_label)?;
        labels_renamed = true;
    }

    if !file_renamed && !labels_renamed {
        println!("{}", tr("No se realizaron cambios", "No changes made"));
    }

    Ok(())
}

pub fn rename_file(paths: &WorkspacePaths, old_name: &str, new_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let old_path = paths.notes_slipbox.join(format!("{old_name}.tex"));
    let new_path = paths.notes_slipbox.join(format!("{new_name}.tex"));

    if new_path.exists() {
        bail!(tr!(
            "El archivo {new_name}.tex ya existe",
            "File {new_name}.tex already exists"
        ));
    }
    if !db.note_exists(old_name)? {
        bail!(tr!(
            "Nota {old_name} no encontrada en la base de datos",
            "Note {old_name} not found in database"
        ));
    }
    if !old_path.exists() {
        bail!(
            "{}",
            tr!(
                "El archivo de la nota {} no existe",
                "Note file {} does not exist",
                old_path.display()
            )
        );
    }

    fs::rename(&old_path, &new_path)?;
    db.rename_note_filename(old_name, new_name)?;
    update_documents_externaldocument(&paths.root.join("notes/documents.tex"), old_name, new_name)?;

    replace_references_in_folder(&paths.notes_slipbox, old_name, new_name)?;
    replace_references_in_folder(&paths.projects, old_name, new_name)?;

    // Remove stale exported artifacts (pdf/markdown) for the old name
    let pdf_path = crate::pdf_output_dir(paths).join(format!("{}.pdf", old_name));
    if pdf_path.exists() {
        fs::remove_file(&pdf_path)?;
    }
    let md_path = export_notes_dir(paths).join(format!("{}.md", old_name));
    if md_path.exists() {
        fs::remove_file(&md_path)?;
    }

    println!("{} {old_name} -> {new_name}", tr("Renombrando", "Renaming"));
    println!(
        "{} {old_name} {} {new_name}",
        tr("Renombrado exitosamente", "Successfully renamed"),
        tr("a", "to")
    );
    Ok(())
}

pub fn rename_label(
    paths: &WorkspacePaths,
    note_name: &str,
    old_label: &str,
    new_label: &str,
) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if !db.note_exists(note_name)? {
        bail!(tr!(
            "Nota {note_name} no encontrada en la base de datos",
            "Note {note_name} not found in database"
        ));
    }

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    let original = fs::read_to_string(&note_path)?;
    let own_label_pat = Regex::new(&format!(r"\\label\{{{}\}}", regex::escape(old_label)))?;
    let updated_note = own_label_pat
        .replace_all(&original, format!(r"\label{{{new_label}}}"))
        .to_string();
    fs::write(&note_path, updated_note)?;

    replace_label_references_in_folder(&paths.notes_slipbox, note_name, old_label, new_label)?;
    replace_label_references_in_folder(&paths.projects, note_name, old_label, new_label)?;

    let _ = synchronize_notes(paths)?;

    println!(
        "{} {old_label} {} {new_label} {} {note_name}",
        tr(
            "Etiqueta renombrada exitosamente de",
            "Successfully renamed label from"
        ),
        tr("a", "to"),
        tr("en", "in")
    );
    Ok(())
}

pub fn remove_note(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;

    let incoming_references = incoming_references_to_note(paths, note_name)?;
    if !incoming_references.is_empty() {
        println!(
            "{} '{note_name}' {}:",
            tr("La nota", "The note"),
            tr("esta referenciada desde", "is referenced from")
        );
        for reference in &incoming_references {
            println!("- {reference}");
        }
        print!(
            "{} [y/N]: ",
            tr("¿Continuar con el borrado?", "Continue with deletion?")
        );
        io::stdout().flush()?;

        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        let answer = answer.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            println!("{}", tr("Borrado cancelado", "Deletion canceled"));
            return Ok(());
        }
    }

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    if note_path.exists() {
        fs::remove_file(&note_path)?;
    }

    remove_externaldocument_line(&paths.root.join("notes/documents.tex"), note_name)?;
    db.delete_note_by_filename(note_name)?;

    println!("{} {note_name}", tr!("Nota eliminada", "Removed note"));
    Ok(())
}

fn incoming_references_to_note(paths: &WorkspacePaths, target_note: &str) -> Result<Vec<String>> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let target_labels = db.labels_for_note(target_note)?;
    let target_labels_set = target_labels.iter().collect::<BTreeSet<_>>();
    let excref_no_label_re = Regex::new(r"\\excref\{([^}]+)\}")?;
    let transclude_re = Regex::new(r"\\transclude(?:\[[^\]]+\])?\{([^}]+)\}")?;
    let mut refs = BTreeSet::new();

    let mut scan_file = |path: &Path, source_label: String| -> Result<()> {
        let content = fs::read_to_string(path)?;
        let parsed = parse_note(&content)?;

        let via_structured_ref = parsed
            .references
            .iter()
            .any(|reference| reference.target_note == target_note);
        let via_excref_without_label = excref_no_label_re
            .captures_iter(&content)
            .any(|caps| caps.get(1).map(|m| m.as_str().trim()) == Some(target_note));
        let via_transclude = transclude_re
            .captures_iter(&content)
            .any(|caps| caps.get(1).map(|m| m.as_str().trim()) == Some(target_note));
        let via_plain_ref = !target_labels_set.is_empty()
            && parsed
                .plain_refs
                .iter()
                .any(|reference| target_labels_set.contains(&reference));

        if source_label != target_note
            && (via_structured_ref || via_excref_without_label || via_transclude || via_plain_ref)
        {
            let title = db
                .note_title_by_filename(&source_label)?
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| source_label.clone());
            refs.insert(format!("{source_label} ({title})"));
        }

        Ok(())
    };

    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        let Some(source_note) = note_stem_from_path(&path) else {
            continue;
        };
        scan_file(&path, source_note)?;
    }

    for entry in fs::read_dir(&paths.projects)? {
        let entry = entry?;
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }
        let Some(project_name) = project_dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let mut tex_files = Vec::new();
        collect_tex_files(&project_dir, &mut tex_files)?;
        for tex_file in tex_files {
            let relative = tex_file
                .strip_prefix(&project_dir)
                .unwrap_or(&tex_file)
                .to_string_lossy()
                .replace('\\', "/");
            let source_label = format!("projects/{project_name}/{relative}");
            scan_file(&tex_file, source_label)?;
        }
    }

    Ok(refs.into_iter().collect())
}

fn update_documents_externaldocument(
    documents_path: &Path,
    old_name: &str,
    new_name: &str,
) -> Result<()> {
    if !documents_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(documents_path)?;
    let pat = Regex::new(&format!(
        r"\\externaldocument\[{}-\]\{{{}\}}",
        regex::escape(old_name),
        regex::escape(old_name)
    ))?;
    let replaced = pat
        .replace_all(
            &content,
            format!(r"\externaldocument[{new_name}-]{{{new_name}}}"),
        )
        .to_string();
    fs::write(documents_path, replaced)?;
    Ok(())
}

fn remove_externaldocument_line(documents_path: &Path, note_name: &str) -> Result<()> {
    if !documents_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(documents_path)?;
    let mut out = Vec::new();
    for line in content.lines() {
        if line.contains(&format!("{{{note_name}}}")) && line.contains("\\externaldocument[") {
            continue;
        }
        out.push(line);
    }
    let mut rebuilt = out.join("\n");
    if !rebuilt.is_empty() {
        rebuilt.push('\n');
    }
    fs::write(documents_path, rebuilt)?;
    Ok(())
}

fn replace_references_in_folder(root: &Path, old_name: &str, new_name: &str) -> Result<()> {
    let patterns = vec![
        (
            Regex::new(&format!(r"\\transclude\{{{}\}}", regex::escape(old_name)))?,
            format!(r"\transclude{{{new_name}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\transclude\[([^\]]+)\]\{{{}\}}",
                regex::escape(old_name)
            ))?,
            format!(r"\transclude[$1]{{{new_name}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\exref\[([^\]]+)\]\{{{}\}}",
                regex::escape(old_name)
            ))?,
            format!(r"\exref[$1]{{{new_name}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\excref\[([^\]]+)\]\{{{}\}}",
                regex::escape(old_name)
            ))?,
            format!(r"\excref[$1]{{{new_name}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\excref\{{{}\}}\{{([^}}]+)\}}",
                regex::escape(old_name)
            ))?,
            format!(r"\excref{{{new_name}}}{{$1}}"),
        ),
        (
            Regex::new(&format!(
                r"\\exhyperref\[([^\]]+)\]\{{{}\}}\{{([^}}]+)\}}",
                regex::escape(old_name)
            ))?,
            format!(r"\exhyperref[$1]{{{new_name}}}{{$2}}"),
        ),
        (
            Regex::new(&format!(
                r"\\exhyperref\{{{}\}}\{{([^}}]+)\}}\{{([^}}]+)\}}",
                regex::escape(old_name)
            ))?,
            format!(r"\exhyperref{{{new_name}}}{{$1}}{{$2}}"),
        ),
        (
            Regex::new(&format!(r"\\ref\{{{}-", regex::escape(old_name)))?,
            format!(r"\ref{{{new_name}-"),
        ),
        (
            Regex::new(&format!(r"\\hyperref\[{}-", regex::escape(old_name)))?,
            format!(r"\hyperref[{new_name}-"),
        ),
    ];

    rewrite_tex_files_recursive(root, &patterns)
}

fn replace_label_references_in_folder(
    root: &Path,
    note_name: &str,
    old_label: &str,
    new_label: &str,
) -> Result<()> {
    let full_old = format!("{note_name}-{old_label}");
    let full_new = format!("{note_name}-{new_label}");

    let patterns = vec![
        // also handle internal references like \ref{defn:old}
        (
            Regex::new(&format!(r"\\ref\{{{}\}}", regex::escape(old_label)))?,
            format!(r"\ref{{{new_label}}}"),
        ),
        (
            Regex::new(&format!(r"\\hyperref\[{}\]", regex::escape(old_label)))?,
            format!(r"\hyperref[{new_label}]"),
        ),
        (
            Regex::new(&format!(r"\\ref\{{{}\}}", regex::escape(&full_old)))?,
            format!(r"\ref{{{full_new}}}"),
        ),
        (
            Regex::new(&format!(r"\\hyperref\[{}\]", regex::escape(&full_old)))?,
            format!(r"\hyperref[{full_new}]"),
        ),
        (
            Regex::new(&format!(
                r"\\excref\[{}\]\{{{}\}}",
                regex::escape(old_label),
                regex::escape(note_name)
            ))?,
            format!(r"\excref[{new_label}]{{{note_name}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\exhyperref\[{}\]\{{{}\}}\{{([^}}]+)\}}",
                regex::escape(old_label),
                regex::escape(note_name)
            ))?,
            format!(r"\exhyperref[{new_label}]{{{note_name}}}{{$1}}"),
        ),
        (
            Regex::new(&format!(
                r"\\excref\{{{}\}}\{{{}\}}",
                regex::escape(note_name),
                regex::escape(old_label)
            ))?,
            format!(r"\excref{{{note_name}}}{{{new_label}}}"),
        ),
        (
            Regex::new(&format!(
                r"\\exhyperref\{{{}\}}\{{{}\}}\{{([^}}]+)\}}",
                regex::escape(note_name),
                regex::escape(old_label)
            ))?,
            format!(r"\exhyperref{{{note_name}}}{{{new_label}}}{{$1}}"),
        ),
    ];

    rewrite_tex_files_recursive(root, &patterns)
}

fn rewrite_tex_files_recursive(root: &Path, patterns: &[(Regex, String)]) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            rewrite_tex_files_recursive(&path, patterns)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("tex") {
            continue;
        }

        let original = fs::read_to_string(&path)?;
        let mut updated = original.clone();
        for (re, replacement) in patterns {
            updated = re.replace_all(&updated, replacement.as_str()).to_string();
        }

        if updated != original {
            fs::write(&path, updated)?;
        }
    }
    Ok(())
}
