use super::*;

/// Render a target (note or project) to PDF via pdflatex.
///
/// Notes inject a "Referenciado en" (referenced-in) section and render from a
/// temp copy; projects render their primary `.tex` file directly.
pub(crate) fn render_pdf(
    paths: &WorkspacePaths,
    target: RenderTarget,
    with_biber: bool,
) -> Result<()> {
    let prepared = match &target {
        RenderTarget::Note(name) => {
            let note_path = target.source_path(paths);
            if !note_path.exists() {
                bail!(
                    "{}: {}",
                    tr("El archivo no existe", "No such file"),
                    note_path.display()
                );
            }

            let output_dir = pdf_output_dir(paths);
            fs::create_dir_all(&output_dir)?;
            fs::canonicalize(&output_dir)?;

            // Ensure template files exist on disk — otherwise TeX will fail silently.
            ensure_template_available_or_suggest_init(paths)?;
            let original_content = fs::read_to_string(&note_path)?;
            // The "Referenciado en" section links to each referencing note via
            // `\externaldocument[source-]{source}` (see documents.tex). Those links
            // only resolve if the referencing notes' .aux exist in the output dir, so
            // ensure them (a single raw pass) before compiling this note.
            let incoming_notes = notes_referencing_target(paths, name)?;
            ensure_backlink_sources(paths, &incoming_notes)?;
            let render_content = inject_referenced_in_section(&original_content, &incoming_notes);

            let temp_dir = ztx_temp_dir(&output_dir)?;
            let temp_filename = format!(".zetteltex-render-{name}.input");
            let temp_path = temp_dir.join(&temp_filename);
            fs::write(&temp_path, render_content)?;

            PreparedRenderInput {
                input_arg: temp_path.to_string_lossy().to_string(),
                cwd: target.source_dir(paths),
                cleanup: vec![temp_path],
            }
        }
        RenderTarget::Project(_) => {
            let project_path = target.source_path(paths);
            if !project_path.exists() {
                bail!(
                    "{}: {}",
                    tr("El archivo no existe", "No such file"),
                    project_path.display()
                );
            }
            let file_name = project_path
                .file_name()
                .with_context(|| {
                    tr!(
                        "no se pudo obtener el nombre del archivo del proyecto",
                        "could not get the project file name"
                    )
                })?
                .to_string_lossy()
                .to_string();
            PreparedRenderInput {
                input_arg: file_name,
                cwd: target.source_dir(paths),
                cleanup: vec![],
            }
        }
    };

    // pdflatex needs 2 passes for \label/\ref and a third one after biber to
    // settle biblatex's citations (with only 2 passes it leaves "Please rerun").
    run_pdflatex_pass(
        paths,
        target.name(),
        prepared.input_arg.as_str(),
        &prepared.cwd,
    )?;
    if with_biber {
        target.run_biber(paths, None)?;
    }
    run_pdflatex_pass(
        paths,
        target.name(),
        prepared.input_arg.as_str(),
        &prepared.cwd,
    )?;
    if with_biber {
        run_pdflatex_pass(
            paths,
            target.name(),
            prepared.input_arg.as_str(),
            &prepared.cwd,
        )?;
    }

    // Keep the temp file for debugging when pdflatex fails.
    for cleanup in prepared.cleanup {
        fs::remove_file(&cleanup)?;
    }

    Ok(())
}

pub(crate) fn render_note_pdf(paths: &WorkspacePaths, name: &str, with_biber: bool) -> Result<()> {
    render_pdf(paths, RenderTarget::Note(name.to_string()), with_biber)
}

pub(crate) fn render_project_pdf(
    paths: &WorkspacePaths,
    name: &str,
    with_biber: bool,
) -> Result<()> {
    render_pdf(paths, RenderTarget::Project(name.to_string()), with_biber)
}

pub(crate) fn ensure_backlink_sources(
    paths: &WorkspacePaths,
    incoming_notes: &[(String, String)],
) -> Result<()> {
    let output_dir = pdf_output_dir(paths);
    for (source, _) in incoming_notes {
        let tex_path = paths.notes_slipbox.join(format!("{source}.tex"));
        if !tex_path.exists() {
            continue;
        }
        let aux_path = output_dir.join(format!("{source}.aux"));
        let pdf_path = output_dir.join(format!("{source}.pdf"));
        let aux_exists = aux_path.exists();
        let pdf_exists = pdf_path.exists();
        let stale = match (
            fs::metadata(&tex_path).and_then(|m| m.modified()).ok(),
            fs::metadata(&aux_path).and_then(|m| m.modified()).ok(),
        ) {
            (Some(tex_mtime), Some(aux_mtime)) => tex_mtime > aux_mtime,
            _ => false,
        };

        if !aux_exists || !pdf_exists || stale {
            // A single raw pass is enough: the only thing we need from the
            // referencing note is its `note` label (the Doc-Start anchor).
            run_pdflatex_pass(
                paths,
                source,
                &tex_path.to_string_lossy(),
                &paths.notes_slipbox,
            )?;
        }
    }
    Ok(())
}

pub(crate) fn run_pdflatex_pass(
    paths: &WorkspacePaths,
    name: &str,
    input_path: &str,
    cwd: &Path,
) -> Result<()> {
    let output_dir = pdf_output_dir(paths);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;

    let mut args = vec![
        "-interaction=nonstopmode".to_string(),
        format!("--jobname={name}"),
        format!("-output-directory={}", output_dir.display()),
    ];
    let config = load_zetteltex_config(paths);
    if config.render.allow_shell_escape {
        args.insert(2, "-shell-escape".to_string());
    }
    args.push(input_path.to_string());

    run_external_tool(
        "pdflatex",
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
        Some(cwd),
        Some(config.render.tool_timeout()),
    )
}

pub(crate) fn pdf_output_dir(paths: &WorkspacePaths) -> PathBuf {
    let config = load_zetteltex_config(paths);
    config
        .render
        .pdf_output_dir
        .as_deref()
        .map(|raw| resolve_config_path(&paths.root, raw))
        .unwrap_or_else(|| paths.root.join("pdf"))
}
