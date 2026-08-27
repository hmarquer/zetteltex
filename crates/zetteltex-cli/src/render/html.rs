use super::*;

pub(crate) const HTML_TEX4HT_MATH_OPTS: &str =
    "pic-m+,pic-equation,pic-eqnarray,pic-array,pic-matrix,pic-align,pic-cases";

/// Render a target (note or project) to HTML via make4ht (tex4ht).
///
/// Notes inject the "Referenciado en" section and HTML overrides, then render
/// from a temp copy; projects render their primary `.tex` file directly.
pub(crate) fn render_html_single_pass(paths: &WorkspacePaths, target: &RenderTarget) -> Result<()> {
    let prepared = match target {
        RenderTarget::Note(name) => {
            let note_path = target.source_path(paths);
            if !note_path.exists() {
                bail!(
                    "{}: {}",
                    tr("El archivo no existe", "No such file"),
                    note_path.display()
                );
            }

            let output_dir = html_output_dir(paths);
            fs::create_dir_all(&output_dir)?;
            fs::canonicalize(&output_dir)?;

            // Ensure template files exist on disk — otherwise make4ht may fail.
            ensure_template_available_or_suggest_init(paths)?;
            let original_content = fs::read_to_string(&note_path)?;
            let incoming_notes = notes_referencing_target(paths, name)?;
            let render_content = inject_referenced_in_section(&original_content, &incoming_notes);
            let render_content = inject_html_overrides(&render_content);

            let temp_dir = ztx_temp_dir(&output_dir)?;
            let temp_filename = format!(".zetteltex-render-{name}.html.tex");
            let temp_path = temp_dir.join(&temp_filename);
            fs::write(&temp_path, &render_content)?;

            let debug_filename = format!(".zetteltex-render-{name}.html.tex.debug");
            let debug_path = temp_dir.join(&debug_filename);
            // Keep a debug copy in case make4ht removes the input on failure.
            fs::write(&debug_path, &render_content)?;

            PreparedRenderInput {
                input_arg: temp_path.to_string_lossy().to_string(),
                cwd: target.source_dir(paths),
                cleanup: vec![temp_path, debug_path],
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
                .context("project file name")?
                .to_string_lossy()
                .to_string();
            PreparedRenderInput {
                input_arg: file_name,
                cwd: target.source_dir(paths),
                cleanup: vec![],
            }
        }
    };

    let output_dir = html_output_dir(paths);
    let output_dir_str = output_dir.to_string_lossy().to_string();

    let mut args = vec![
        "--format".to_string(),
        "html5+svg".to_string(),
        "--output-dir".to_string(),
        output_dir_str,
        "--jobname".to_string(),
        target.name().to_string(),
    ];
    if load_zetteltex_config(paths).render.allow_shell_escape {
        args.push("--shell-escape".to_string());
    }
    args.push(prepared.input_arg.clone());
    args.push(HTML_TEX4HT_MATH_OPTS.to_string());

    let render_result = run_external_tool(
        "make4ht",
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
        Some(&prepared.cwd),
    );

    match render_result {
        Ok(_) => {
            for cleanup in prepared.cleanup {
                fs::remove_file(&cleanup)?;
            }
            Ok(())
        }
        Err(err) => Err(err),
    }
}

pub(crate) fn render_note_html_single_pass(paths: &WorkspacePaths, name: &str) -> Result<()> {
    render_html_single_pass(paths, &RenderTarget::Note(name.to_string()))
}

pub(crate) fn render_project_html_single_pass(paths: &WorkspacePaths, name: &str) -> Result<()> {
    render_html_single_pass(paths, &RenderTarget::Project(name.to_string()))
}

pub(crate) fn inject_html_overrides(note_content: &str) -> String {
    let note_content = note_content.replace("\\[", "$$").replace("\\]", "$$");
    let injection = r#"
% ztx html overrides
\ifx\HCode\UnDeFiNeD
\else
\makeatletter
\AtBeginDocument{%
    \def\[{$$}%
    \def\]{$$}%
    \renewcommand{\text}[1]{\mbox{#1}}%
    \renewcommand{\href}[2]{#2}%
    \renewcommand{\hyperref}[2][]{#2}%
    \renewcommand{\exhyperref}[3][]{#3}%
    \renewcommand{\excref}[2][]{\texttt{#2}}%
    \renewcommand{\exref}[2][]{\texttt{#2}}%
}
\makeatother
\fi
"#;

    if let Some(idx) = note_content.find("\\begin{document}") {
        let mut out = String::with_capacity(note_content.len() + injection.len());
        out.push_str(&note_content[..idx]);
        out.push_str(injection);
        out.push_str(&note_content[idx..]);
        out
    } else {
        let mut out = String::with_capacity(note_content.len() + injection.len());
        out.push_str(injection);
        out.push_str(&note_content);
        out
    }
}

pub(crate) fn html_output_dir(paths: &WorkspacePaths) -> PathBuf {
    let config = load_zetteltex_config(paths);
    config
        .render
        .html_output_dir
        .as_deref()
        .map(|raw| resolve_config_path(&paths.root, raw))
        .unwrap_or_else(|| paths.root.join("html"))
}
