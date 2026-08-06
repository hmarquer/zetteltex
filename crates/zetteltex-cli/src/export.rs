use super::*;

pub(crate) fn export_projects_dir(paths: &WorkspacePaths) -> PathBuf {
    let config = load_zetteltex_config(paths);
    let vault = config
        .export
        .obsidian_vault
        .as_deref()
        .map(|raw| resolve_config_path(&paths.root, raw))
        .unwrap_or_else(|| paths.root.join("jabberwocky"));

    let subdir = config
        .export
        .projects_subdir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("latex").join("asignaturas"));

    vault.join(subdir)
}

pub(crate) fn export_notes_dir(paths: &WorkspacePaths) -> PathBuf {
    let config = load_zetteltex_config(paths);
    let vault = config
        .export
        .obsidian_vault
        .as_deref()
        .map(|raw| resolve_config_path(&paths.root, raw))
        .unwrap_or_else(|| paths.root.join("jabberwocky"));

    let subdir = config
        .export
        .notes_subdir
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("latex").join("zettelkasten"));

    vault.join(subdir)
}

pub(crate) fn subject_tags_for_note(paths: &WorkspacePaths, note_name: &str) -> Result<Vec<String>> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let projects = db.list_note_projects(note_name)?;
    let mut tags = std::collections::BTreeSet::new();
    for p in projects {
        let clean = clean_project_tag(&p.project_name);
        if clean.is_empty() {
            continue;
        }
        let source = p.source_file.trim_end_matches(".tex");
        tags.insert(format!("{clean}/{source}"));
    }
    Ok(tags.into_iter().collect())
}

pub(crate) fn to_md(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    let content = fs::read_to_string(&note_path)?;

    let excref_re = Regex::new(r"\\excref\{([^}]+)\}\{([^}]+)\}")?;
    let exref_re = Regex::new(r"\\exref\[([^\]]+)\]\{([^}]+)\}")?;
    let exhyperref_re =
        Regex::new(r"\\exhyperref(?:\[[^\]]+\])?\{([^}]+)\}\{([^}]+)\}\{([^}]*)\}")?;

    let out1 = excref_re
        .replace_all(&content, |caps: &regex::Captures<'_>| {
            format!("[[{}#{}]]", &caps[1], &caps[2])
        })
        .to_string();
    let out2 = exref_re
        .replace_all(&out1, |caps: &regex::Captures<'_>| {
            format!("[[{}#{}]]", &caps[2], &caps[1])
        })
        .to_string();
    let out3 = exhyperref_re
        .replace_all(&out2, |caps: &regex::Captures<'_>| {
            format!("[[{}#{}|{}]]", &caps[1], &caps[2], &caps[3])
        })
        .to_string();

    let markdown_dir = paths.root.join("markdown");
    fs::create_dir_all(&markdown_dir)?;
    let out_path = markdown_dir.join(format!("{note_name}.md"));
    fs::write(&out_path, out3)?;
    println!("Markdown exported to {}", out_path.display());
    Ok(())
}

pub(crate) fn export_note_markdown_file(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    if !db.note_exists(note_name)? {
        bail!("Note {note_name} not found in database");
    }

    let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
    let content = fs::read_to_string(&note_path)?;
    let parsed = parse_note(&content)?;

    let out_dir = export_notes_dir(paths);
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join(format!("{note_name}.md"));

    let meta = db.note_metadata_by_filename(note_name)?;
    let title = meta
        .as_ref()
        .and_then(|m| m.title.as_deref())
        .filter(|s| !s.trim().is_empty())
        .map(String::from)
        .or_else(|| extract_title_from_tex_content(&content))
        .unwrap_or_else(|| note_name.to_string());
    let tags = subject_tags_for_note(paths, note_name)?;
    let references = parsed
        .references
        .iter()
        .map(|r| r.target_note.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let backlinks = db.notes_referencing_note(note_name)?;
    let citations = db.citations_for_note(note_name)?;
    let labels = db.labels_for_note(note_name)?;
    let projects = db
        .list_note_projects(note_name)?
        .into_iter()
        .map(|p| p.project_name)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let keywords = extract_keywords_from_tex_content(&content);

    let mut out = String::new();
    out.push_str("---\n");
    push_frontmatter_str(&mut out, "title", Some(&title));
    push_frontmatter_str(&mut out, "filename", Some(note_name));
    if let Some(m) = &meta {
        push_frontmatter_str(&mut out, "created", m.created.as_deref());
        push_frontmatter_str(&mut out, "last_edit_date", m.last_edit_date.as_deref());
        push_frontmatter_str(
            &mut out,
            "last_build_date_pdf",
            m.last_build_date_pdf.as_deref(),
        );
        push_frontmatter_str(
            &mut out,
            "last_build_date_html",
            m.last_build_date_html.as_deref(),
        );
    }
    push_frontmatter_list(&mut out, "labels", &labels);
    push_frontmatter_list(&mut out, "references", &references);
    push_frontmatter_list(&mut out, "backlinks", &backlinks);
    push_frontmatter_list(&mut out, "citations", &citations);
    push_frontmatter_list(&mut out, "projects", &projects);
    if !tags.is_empty() {
        out.push_str("tags:\n");
        for tag in &tags {
            out.push_str(&format!("  - {tag}\n"));
        }
    }
    out.push_str("---\n\n");

    out.push_str(&format!("[[{note_name}.pdf]]\n"));
    out.push_str(&format!("![[{note_name}.pdf]]\n\n"));

    if !references.is_empty() {
        out.push_str("## Referencias\n");
        for r in &references {
            out.push_str(&format!("- [{r}](./{r}.md)\n"));
        }
        out.push('\n');
    }

    if !keywords.is_empty() {
        out.push_str("## Etiquetas\n");
        for (k, txt) in keywords {
            out.push_str(&format!("#{k} {txt}\n"));
        }
    }

    fs::write(&out_path, out)?;
    Ok(())
}

pub(crate) fn export_markdown(paths: &WorkspacePaths, note_name: &str) -> Result<()> {
    println!(
        "Plan export_markdown: nota='{}' | sync=true | salida={} ",
        note_name,
        export_notes_dir(paths).display()
    );
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    export_note_markdown_file(paths, note_name)
}

pub(crate) fn export_project_markdown_file(
    paths: &WorkspacePaths,
    project_name: &str,
) -> Result<()> {
    let project_dir = paths.projects.join(project_name);
    let main_tex = project_dir.join(format!("{project_name}.tex"));
    if !main_tex.exists() {
        bail!("Project main tex not found: {}", main_tex.display());
    }

    let db = init_database(&paths.root.join("slipbox.db"))?;
    let inclusions = db.list_project_inclusions_by_name(project_name)?;
    let content = fs::read_to_string(&main_tex)?;

    let out_dir = export_projects_dir(paths);
    fs::create_dir_all(&out_dir)?;
    let out_path = out_dir.join(format!("{project_name}.md"));

    let title =
        extract_title_from_tex_content(&content).unwrap_or_else(|| project_name.to_string());
    let clean_project = clean_project_tag(project_name);
    let keywords = extract_keywords_from_tex_content(&content);
    let meta = db.project_metadata_by_name(project_name)?;
    let inclusion_names = inclusions
        .iter()
        .map(|inc| inc.note_filename.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let mut out = String::new();
    out.push_str("---\n");
    push_frontmatter_str(&mut out, "title", Some(&title));
    push_frontmatter_str(&mut out, "name", Some(project_name));
    if let Some(m) = &meta {
        push_frontmatter_str(&mut out, "created", m.created.as_deref());
        push_frontmatter_str(&mut out, "last_edit_date", m.last_edit_date.as_deref());
        push_frontmatter_str(
            &mut out,
            "last_build_date_pdf",
            m.last_build_date_pdf.as_deref(),
        );
        push_frontmatter_str(
            &mut out,
            "last_build_date_html",
            m.last_build_date_html.as_deref(),
        );
    }
    push_frontmatter_list(&mut out, "inclusions", &inclusion_names);
    if !clean_project.is_empty() {
        out.push_str("tags:\n");
        out.push_str(&format!("  - {}\n", clean_project));
    }
    out.push_str("---\n\n");

    out.push_str(&format!("[[{project_name}.pdf]]\n"));
    out.push_str(&format!("![[{project_name}.pdf]]\n\n"));

    if !inclusions.is_empty() {
        out.push_str("## Notas incluidas\n");
        let mut current_source = String::new();
        for inc in inclusions {
            let source_base = inc.source_file.trim_end_matches(".tex");
            if source_base != current_source {
                out.push_str(&format!("\n### {}\n", source_base));
                current_source = source_base.to_string();
            }
            out.push_str(&format!(
                "- [{}](./{}.md)\n",
                inc.note_filename, inc.note_filename
            ));
        }
        out.push('\n');
    }

    if !keywords.is_empty() {
        out.push_str("## Etiquetas\n");
        for (k, txt) in keywords {
            out.push_str(&format!("#{k} {txt}\n"));
        }
    }

    fs::write(&out_path, out)?;
    Ok(())
}

pub(crate) fn export_project_markdown(paths: &WorkspacePaths, project_name: &str) -> Result<()> {
    println!(
        "Plan export_project_markdown: proyecto='{}' | sync=true | salida={}",
        project_name,
        export_projects_dir(paths).display()
    );
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    export_project_markdown_file(paths, project_name)
}

pub(crate) fn export_all_notes_markdown(paths: &WorkspacePaths) -> Result<()> {
    let note_names: Vec<String> = fs::read_dir(&paths.notes_slipbox)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter_map(|path| note_stem_from_path(&path))
        .collect();

    println!(
        "Plan export_all_notes_markdown: notas={} | sync=true | salida={}",
        note_names.len(),
        export_notes_dir(paths).display()
    );

    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;

    let mut count = 0usize;
    for name in &note_names {
        export_note_markdown_file(paths, name)?;
        count += 1;
    }

    println!(
        "Exported {count} note(s) to {}",
        export_notes_dir(paths).display()
    );
    Ok(())
}

pub(crate) fn export_all_projects_markdown(paths: &WorkspacePaths) -> Result<()> {
    let _ = synchronize_notes(paths)?;
    let _ = synchronize_projects(paths)?;
    let db = init_database(&paths.root.join("slipbox.db"))?;

    let projects = db.list_projects()?;
    println!(
        "Plan export_all_projects_markdown: proyectos={} | sync=true | salida={}",
        projects.len(),
        export_projects_dir(paths).display()
    );

    let mut count = 0usize;
    for p in projects {
        export_project_markdown_file(paths, &p.name)?;
        count += 1;
    }

    println!(
        "Exported {count} project(s) to {}",
        export_projects_dir(paths).display()
    );
    Ok(())
}

pub(crate) fn export_all_markdown(paths: &WorkspacePaths) -> Result<()> {
    println!(
        "Plan export_all_markdown: notas->{} | proyectos->{} | sync=true",
        export_notes_dir(paths).display(),
        export_projects_dir(paths).display()
    );
    export_all_notes_markdown(paths)?;
    export_all_projects_markdown(paths)?;
    Ok(())
}

pub(crate) fn export_draft(paths: &WorkspacePaths, input_file: &str, output_file: &str) -> Result<()> {
    let input_path = resolve_workspace_path(paths, input_file);
    if !input_path.exists() {
        bail!("Input file not found: {}", input_path.display());
    }

    let output_path = resolve_workspace_path(paths, output_file);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let execute_re = Regex::new(r"\\ExecuteMetaData\[([^\]]+)\]\{([^}]+)\}")?;
    let mut output = String::new();
    let input_parent = input_path.parent().unwrap_or(paths.root.as_path());

    for raw_line in fs::read_to_string(&input_path)?.lines() {
        let line_without_exec = execute_re.replace_all(raw_line, "").to_string();
        output.push_str(line_without_exec.trim());
        output.push('\n');

        for caps in execute_re.captures_iter(raw_line) {
            let import_file = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
            let tag = caps.get(2).map(|m| m.as_str()).unwrap_or("note");

            let mut import_path = PathBuf::from(import_file);
            if !import_path.is_absolute() {
                let candidate = input_parent.join(&import_path);
                import_path = if candidate.exists() {
                    candidate
                } else {
                    paths.root.join(&import_path)
                };
            }

            let import_content = fs::read_to_string(&import_path)?;
            if let Some(block) = extract_tagged_block(&import_content, tag)? {
                output.push_str(block.trim());
                output.push('\n');
            } else {
                bail!(
                    "Tag <*{}>...</{}> not found in {}",
                    tag,
                    tag,
                    import_path.display()
                );
            }
        }
    }

    fs::write(output_path, output)?;
    Ok(())
}

fn clean_project_tag(project_name: &str) -> String {
    let without_prefix = project_name
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
        .trim_start_matches('-');
    without_prefix.to_string()
}

fn push_frontmatter_str(out: &mut String, key: &str, value: Option<&str>) {
    if let Some(v) = value {
        let trimmed = v.trim();
        if !trimmed.is_empty() {
            out.push_str(&format!("{key}: '{}'\n", trimmed.replace('\'', "''")));
        }
    }
}

fn push_frontmatter_list(out: &mut String, key: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    out.push_str(&format!("{key}:\n"));
    for v in values {
        out.push_str(&format!("  - {v}\n"));
    }
}

fn extract_keywords_from_tex_content(content: &str) -> Vec<(String, String)> {
    let keys = [
        "TODO:",
        "FIXME:",
        "DEMOSTRACION",
        "DEMOSTRACIÓN",
        "ORDENAR",
        "COMPLETAR",
        "EJERCICIO",
        "REVISAR",
        "FALTA",
    ];

    let mut out = Vec::new();
    for line in content.lines() {
        for key in keys {
            if let Some(idx) = line.find(key) {
                let txt = line[idx + key.len()..].trim().to_string();
                out.push((key.trim_end_matches(':').to_string(), txt));
            }
        }
    }
    out
}

pub(crate) fn export_project(
    paths: &WorkspacePaths,
    project_folder: &str,
    texfile: Option<&str>,
) -> Result<()> {
    let texfile = texfile
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{project_folder}.tex"));

    let input_path = paths.projects.join(project_folder).join(&texfile);
    if !input_path.exists() {
        bail!("Project file not found: {}", input_path.display());
    }

    let output_dir = paths.projects.join(project_folder).join("standalone");
    fs::create_dir_all(&output_dir)?;
    let output_path = output_dir.join(&texfile);

    let transclude_re = Regex::new(r"\\transclude(?:\[([^\]]+)\])?\{([^}]+)\}")?;
    let mut output = String::new();

    for raw_line in fs::read_to_string(&input_path)?.lines() {
        let line_without_transcludes = transclude_re.replace_all(raw_line, "").to_string();
        output.push_str(line_without_transcludes.trim());
        output.push('\n');

        for caps in transclude_re.captures_iter(raw_line) {
            let tag = caps
                .get(1)
                .map(|m| m.as_str().trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "note".to_string());
            let note_name = caps.get(2).map(|m| m.as_str().trim()).unwrap_or_default();
            let note_path = paths.notes_slipbox.join(format!("{note_name}.tex"));
            let note_content = fs::read_to_string(&note_path)?;

            if let Some(block) = extract_tagged_block(&note_content, &tag)? {
                output.push_str(block.trim());
                output.push('\n');
            } else {
                bail!(
                    "Tag <*{}>...</{}> not found in {}",
                    tag,
                    tag,
                    note_path.display()
                );
            }
        }
    }

    fs::write(output_path, output)?;
    Ok(())
}
