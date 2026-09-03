use super::*;
use crate::i18n::tr;
use anyhow::Context;

mod engine;
mod html;
mod pdf;
mod progress;
pub(crate) use engine::*;
pub(crate) use html::*;
pub(crate) use pdf::*;
pub(crate) use progress::*;

/// A render target: either an atomic note in `notes/slipbox/` or a project in
/// `projects/<name>/`. Encapsulates everything that differs between the two so
/// that the PDF/HTML orchestration (and citation detection) is shared.
pub(crate) enum RenderTarget {
    Note(String),
    Project(String),
}

/// The input handed to the external TeX/HTML engine for a render pass.
struct PreparedRenderInput {
    /// Argument to pass to pdflatex/make4ht (a temp path or the source file name).
    input_arg: String,
    /// Working directory for the external tool.
    cwd: PathBuf,
    /// Temp/debug files to delete after a successful run.
    cleanup: Vec<PathBuf>,
}

impl RenderTarget {
    fn name(&self) -> &str {
        match self {
            Self::Note(name) | Self::Project(name) => name,
        }
    }

    fn source_dir(&self, paths: &WorkspacePaths) -> PathBuf {
        match self {
            Self::Note(_) => paths.notes_slipbox.clone(),
            Self::Project(name) => paths.projects.join(name),
        }
    }

    fn source_path(&self, paths: &WorkspacePaths) -> PathBuf {
        self.source_dir(paths).join(format!("{}.tex", self.name()))
    }

    /// Detect citations using the real parser (`parse_note`) for both notes and
    /// projects, so the two paths can never diverge.
    fn contains_citations(&self, paths: &WorkspacePaths) -> Result<bool> {
        let content = fs::read_to_string(self.source_path(paths))?;
        let parsed = parse_note(&content);
        Ok(!parsed.citations.is_empty())
    }

    /// Run Biber in the correct working directory for this target.
    fn run_biber(&self, paths: &WorkspacePaths, folder: Option<&str>) -> Result<()> {
        match self {
            Self::Note(_) => run_biber_cmd(paths, self.name(), folder),
            Self::Project(_) => run_biber_project_cmd(paths, self.name(), folder),
        }
    }
}

pub(crate) fn render_note_cmd(
    paths: &WorkspacePaths,
    name: &str,
    format: &str,
    with_biber: bool,
) -> Result<()> {
    let target = RenderTarget::Note(name.to_string());
    let auto_biber = with_biber || target.contains_citations(paths)?;
    let motor = render_motor(format)?;
    let passes = render_pass_count(format, auto_biber)?;

    match format {
        "pdf" => {
            println!(
                "{}: {}='{}' | formato={} | motor={} | pasadas={} | biber={} | salida={}",
                tr("Plan render", "Render plan"),
                tr("nota", "note"),
                name,
                format,
                motor,
                passes,
                auto_biber,
                pdf_output_dir(paths).display()
            );

            render_pdf(paths, target, auto_biber)?;

            let db = init_database(&paths.root.join("slipbox.db"))?;
            db.set_note_last_build_date_pdf(name, Utc::now())?;
            Ok(())
        }
        "html" => {
            let output_dir = html_output_dir(paths);
            println!(
                "{}: {}='{}' | formato={} | motor={} | pasadas={} | biber={} | salida={}",
                tr("Plan render", "Render plan"),
                tr("nota", "note"),
                name,
                format,
                motor,
                passes,
                auto_biber,
                output_dir.display()
            );

            render_html_single_pass(paths, &target)?;

            if auto_biber {
                let output_dir_str = output_dir.to_string_lossy().to_string();
                target.run_biber(paths, Some(output_dir_str.as_str()))?;
            }

            render_html_single_pass(paths, &target)?;

            postprocess_html_output(paths)?;

            let db = init_database(&paths.root.join("slipbox.db"))?;
            db.set_note_last_build_date_html(name, Utc::now())?;
            Ok(())
        }
        _ => bail!(tr!(
            "Formato no soportado: {format}",
            "Unsupported format: {format}"
        )),
    }
}

pub(crate) fn render_project_cmd(
    paths: &WorkspacePaths,
    name: &str,
    format: &str,
    with_biber: bool,
) -> Result<()> {
    let target = RenderTarget::Project(name.to_string());
    let auto_biber = with_biber || target.contains_citations(paths)?;
    let motor = render_motor(format)?;
    let passes = render_pass_count(format, auto_biber)?;

    match format {
        "pdf" => {
            println!(
                "{}: {}='{}' | formato={} | motor={} | pasadas={} | biber={} | salida={}",
                tr("Plan render", "Render plan"),
                tr("proyecto", "project"),
                name,
                format,
                motor,
                passes,
                auto_biber,
                pdf_output_dir(paths).display()
            );

            render_pdf(paths, target, auto_biber)?;

            let db = init_database(&paths.root.join("slipbox.db"))?;
            db.set_project_last_build_date_pdf(name, Utc::now())?;
            Ok(())
        }
        "html" => {
            let output_dir = html_output_dir(paths);
            println!(
                "{}: {}='{}' | formato={} | motor={} | pasadas={} | biber={} | salida={}",
                tr("Plan render", "Render plan"),
                tr("proyecto", "project"),
                name,
                format,
                motor,
                passes,
                auto_biber,
                output_dir.display()
            );

            render_html_single_pass(paths, &target)?;

            if auto_biber {
                let output_dir_str = output_dir.to_string_lossy().to_string();
                target.run_biber(paths, Some(output_dir_str.as_str()))?;
            }

            render_html_single_pass(paths, &target)?;

            postprocess_html_output(paths)?;

            let db = init_database(&paths.root.join("slipbox.db"))?;
            db.set_project_last_build_date_html(name, Utc::now())?;
            Ok(())
        }
        _ => bail!(tr!(
            "Formato no soportado: {format}",
            "Unsupported format: {format}"
        )),
    }
}

pub(crate) fn render_all_notes_cmd(
    paths: &WorkspacePaths,
    format: &str,
    workers: usize,
) -> Result<()> {
    match format {
        "pdf" => {
            let db = init_database(&paths.root.join("slipbox.db"))?;
            let mut note_names = Vec::new();
            for entry in fs::read_dir(&paths.notes_slipbox)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(name) = note_stem_from_path(&path) {
                    note_names.push(name);
                }
            }

            let mut with_citations = HashMap::new();
            for name in &note_names {
                with_citations.insert(
                    name.clone(),
                    RenderTarget::Note(name.clone()).contains_citations(paths)?,
                );
            }
            let notes_with_biber = with_citations.values().filter(|v| **v).count();

            println!(
                "{}: notas={} | workers={} | formato={} | motor={} | pasadas=2/3 | con_biber={} | salida={}",
                tr("Plan render_all", "Render all plan"),
                note_names.len(),
                workers.max(1).min(note_names.len().max(1)),
                format,
                render_motor(format)?,
                notes_with_biber,
                pdf_output_dir(paths).display()
            );

            // Warm up: ensure every referencing note's .aux/.pdf exist before
            // the workers start, so the "Referenciado en" backlinks resolve
            // even for notes that are being recompiled in parallel. Built in a
            // single O(n) pass instead of re-scanning all notes per target.
            let incoming_index = build_incoming_references_index(paths)?;
            for name in &note_names {
                if let Some(incoming) = incoming_index.get(name) {
                    ensure_backlink_sources(paths, incoming)?;
                }
            }

            let paths_render = paths.clone();
            let citations_render = with_citations.clone();
            run_parallel_render_with_progress(
                tr("Render notas", "Render notes"),
                note_names.clone(),
                workers,
                move |name| {
                    let use_biber = citations_render.get(name).copied().unwrap_or(false);
                    render_note_pdf(&paths_render, name, use_biber)?;
                    Ok(())
                },
            )?;

            for name in &note_names {
                db.set_note_last_build_date_pdf(name, Utc::now())?;
            }

            Ok(())
        }
        "html" => {
            let db = init_database(&paths.root.join("slipbox.db"))?;
            let mut note_names = Vec::new();
            for entry in fs::read_dir(&paths.notes_slipbox)? {
                let entry = entry?;
                let path = entry.path();
                if let Some(name) = note_stem_from_path(&path) {
                    note_names.push(name);
                }
            }

            let mut with_citations = HashMap::new();
            for name in &note_names {
                with_citations.insert(
                    name.clone(),
                    RenderTarget::Note(name.clone()).contains_citations(paths)?,
                );
            }

            let notes_with_biber = with_citations.values().filter(|v| **v).count();
            let output_dir = html_output_dir(paths);
            let output_dir_str = output_dir.to_string_lossy().to_string();
            println!(
                "{}: notas={} | workers={} | formato={} | motor={} | pasadas=2 | con_biber={} | salida={}",
                tr("Plan render_all", "Render all plan"),
                note_names.len(),
                workers.max(1).min(note_names.len().max(1)),
                format,
                render_motor(format)?,
                notes_with_biber,
                output_dir.display()
            );

            let paths_pass1 = paths.clone();
            let output_dir_pass1 = output_dir_str.clone();
            run_parallel_render_with_progress(
                tr("Render notas · pasada 1/2", "Render notes · pass 1/2"),
                note_names.clone(),
                workers,
                move |name| {
                    render_note_html_single_pass(&paths_pass1, name)?;
                    if with_citations.get(name).copied().unwrap_or(false) {
                        run_biber_cmd(&paths_pass1, name, Some(output_dir_pass1.as_str()))?;
                    }
                    Ok(())
                },
            )?;

            let paths_pass2 = paths.clone();
            run_parallel_render_with_progress(
                tr("Render notas · pasada 2/2", "Render notes · pass 2/2"),
                note_names.clone(),
                workers,
                move |name| {
                    render_note_html_single_pass(&paths_pass2, name)?;
                    Ok(())
                },
            )?;

            postprocess_html_output(paths)?;

            for name in &note_names {
                db.set_note_last_build_date_html(name, Utc::now())?;
            }

            Ok(())
        }
        _ => bail!(tr!(
            "Formato no soportado: {format}",
            "Unsupported format: {format}"
        )),
    }
}

pub(crate) fn render_all_projects_cmd(
    paths: &WorkspacePaths,
    format: &str,
    workers: usize,
) -> Result<()> {
    match format {
        "pdf" => {
            let db = init_database(&paths.root.join("slipbox.db"))?;
            let mut project_names = Vec::new();
            for entry in fs::read_dir(&paths.projects)? {
                let entry = entry?;
                let dir = entry.path();
                if !dir.is_dir() {
                    continue;
                }
                if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                    let main = dir.join(format!("{name}.tex"));
                    if main.exists() {
                        project_names.push(name.to_string());
                    }
                }
            }

            let mut with_citations = HashMap::new();
            for name in &project_names {
                with_citations.insert(
                    name.clone(),
                    RenderTarget::Project(name.clone()).contains_citations(paths)?,
                );
            }
            let projects_with_biber = with_citations.values().filter(|v| **v).count();

            println!(
                "{}: proyectos={} | workers={} | formato={} | motor={} | pasadas=2/3 | con_biber={} | salida={}",
                tr("Plan render_all_projects", "Render all projects plan"),
                project_names.len(),
                workers.max(1).min(project_names.len().max(1)),
                format,
                render_motor(format)?,
                projects_with_biber,
                pdf_output_dir(paths).display()
            );

            let paths_render = paths.clone();
            let citations_render = with_citations.clone();
            run_parallel_render_with_progress(
                tr("Render proyectos", "Render projects"),
                project_names.clone(),
                workers,
                move |name| {
                    let use_biber = citations_render.get(name).copied().unwrap_or(false);
                    render_project_pdf(&paths_render, name, use_biber)?;
                    Ok(())
                },
            )?;

            for name in &project_names {
                db.set_project_last_build_date_pdf(name, Utc::now())?;
            }

            Ok(())
        }
        "html" => {
            let db = init_database(&paths.root.join("slipbox.db"))?;
            let mut project_names = Vec::new();
            for entry in fs::read_dir(&paths.projects)? {
                let entry = entry?;
                let dir = entry.path();
                if !dir.is_dir() {
                    continue;
                }
                if let Some(name) = dir.file_name().and_then(|s| s.to_str()) {
                    let main = dir.join(format!("{name}.tex"));
                    if main.exists() {
                        project_names.push(name.to_string());
                    }
                }
            }

            let output_dir = html_output_dir(paths);
            let output_dir_str = output_dir.to_string_lossy().to_string();

            let mut with_citations = HashMap::new();
            for name in &project_names {
                with_citations.insert(
                    name.clone(),
                    RenderTarget::Project(name.clone()).contains_citations(paths)?,
                );
            }
            let projects_with_biber = with_citations.values().filter(|v| **v).count();

            println!(
                "{}: proyectos={} | workers={} | formato={} | motor={} | pasadas=2 | con_biber={} | salida={}",
                tr("Plan render_all_projects", "Render all projects plan"),
                project_names.len(),
                workers.max(1).min(project_names.len().max(1)),
                format,
                render_motor(format)?,
                projects_with_biber,
                output_dir.display()
            );

            let paths_pass1 = paths.clone();
            let output_dir_pass1 = output_dir_str.clone();
            let citations_pass1 = with_citations.clone();
            run_parallel_render_with_progress(
                tr(
                    "Render proyectos · pasada 1/2",
                    "Render projects · pass 1/2",
                ),
                project_names.clone(),
                workers,
                move |name| {
                    render_project_html_single_pass(&paths_pass1, name)?;
                    if citations_pass1.get(name).copied().unwrap_or(false) {
                        run_biber_project_cmd(&paths_pass1, name, Some(output_dir_pass1.as_str()))?;
                    }
                    Ok(())
                },
            )?;

            let paths_pass2 = paths.clone();
            run_parallel_render_with_progress(
                tr(
                    "Render proyectos · pasada 2/2",
                    "Render projects · pass 2/2",
                ),
                project_names.clone(),
                workers,
                move |name| {
                    render_project_html_single_pass(&paths_pass2, name)?;
                    Ok(())
                },
            )?;

            postprocess_html_output(paths)?;

            for name in &project_names {
                db.set_project_last_build_date_html(name, Utc::now())?;
            }

            Ok(())
        }
        _ => bail!(tr!(
            "Formato no soportado: {format}",
            "Unsupported format: {format}"
        )),
    }
}

pub(crate) fn render_updates_cmd(
    paths: &WorkspacePaths,
    format: &str,
    workers: usize,
) -> Result<()> {
    match format {
        "pdf" => {
            println!(
                "{}",
                tr!(
                    "Preparando render_updates: sincronizando indices...",
                    "Preparing render_updates: synchronizing indexes..."
                )
            );
            let _ = run_with_sqlite_lock_retry("synchronize notes", || synchronize_notes(paths))?;
            let _ =
                run_with_sqlite_lock_retry("synchronize projects", || synchronize_projects(paths))?;

            let db = Arc::new(Mutex::new(run_with_sqlite_lock_retry(
                "open database",
                || init_database(&paths.root.join("slipbox.db")).map_err(anyhow::Error::from),
            )?));
            let notes = {
                let db = db.lock().unwrap();
                db.notes_needing_render()?
                    .into_iter()
                    .filter(|n| !is_render_temp_note_name(n))
                    .filter(|n| paths.notes_slipbox.join(format!("{n}.tex")).exists())
                    .collect::<Vec<_>>()
            };
            let projects = {
                let db = db.lock().unwrap();
                db.projects_needing_render()?
            };

            if notes.is_empty() && projects.is_empty() {
                println!(
                    "{}",
                    tr!(
                        "No hay elementos pendientes de renderizado.",
                        "No items pending render."
                    )
                );
                return Ok(());
            }

            println!(
                "{}: notas={} | proyectos={} | workers={} | formato={} | motor={} | pasadas=2/3 | salida={}",
                tr("Plan render_updates", "Render updates plan"),
                notes.len(),
                projects.len(),
                workers.max(1),
                format,
                render_motor(format)?,
                pdf_output_dir(paths).display()
            );

            // Warm up: ensure the backlink sources for the stale notes exist
            // before the parallel phase (see render_all_notes_cmd). Built in a
            // single O(n) pass instead of re-scanning all notes per target.
            let incoming_index = build_incoming_references_index(paths)?;
            for name in &notes {
                if let Some(incoming) = incoming_index.get(name) {
                    ensure_backlink_sources(paths, incoming)?;
                }
            }

            let paths_notes = paths.clone();
            let db_notes = db.clone();
            run_parallel_render_with_progress(
                tr("Render updates · notas", "Render updates · notes"),
                notes.clone(),
                workers,
                move |name| {
                    render_note_pdf(&paths_notes, name, false)?;
                    let now = Utc::now();
                    run_with_sqlite_lock_retry("update note last_build_date_pdf", || {
                        db_notes
                            .lock()
                            .unwrap()
                            .set_note_last_build_date_pdf(name, now)
                            .map_err(anyhow::Error::from)
                    })?;
                    Ok(())
                },
            )?;

            let paths_projects = paths.clone();
            let db_projects = db.clone();
            run_parallel_render_with_progress(
                tr("Render updates · proyectos", "Render updates · projects"),
                projects.clone(),
                workers,
                move |name| {
                    render_project_pdf(&paths_projects, name, false)?;
                    let now = Utc::now();
                    run_with_sqlite_lock_retry("update project last_build_date_pdf", || {
                        db_projects
                            .lock()
                            .unwrap()
                            .set_project_last_build_date_pdf(name, now)
                            .map_err(anyhow::Error::from)
                    })?;
                    Ok(())
                },
            )?;

            Ok(())
        }
        "html" => {
            println!(
                "{}",
                tr!(
                    "Preparando render_updates: sincronizando indices...",
                    "Preparing render_updates: synchronizing indexes..."
                )
            );
            let _ = run_with_sqlite_lock_retry("synchronize notes", || synchronize_notes(paths))?;
            let _ =
                run_with_sqlite_lock_retry("synchronize projects", || synchronize_projects(paths))?;

            let db = Arc::new(Mutex::new(run_with_sqlite_lock_retry(
                "open database",
                || init_database(&paths.root.join("slipbox.db")).map_err(anyhow::Error::from),
            )?));
            let notes = {
                let db = db.lock().unwrap();
                db.notes_needing_render_html()?
                    .into_iter()
                    .filter(|n| !is_render_temp_note_name(n))
                    .filter(|n| paths.notes_slipbox.join(format!("{n}.tex")).exists())
                    .map(|n| (n.clone(), db.note_has_citations(&n).unwrap_or(false)))
                    .collect::<Vec<_>>()
            };
            let projects = {
                let db = db.lock().unwrap();
                db.projects_needing_render_html()?
            };

            let mut note_names = Vec::new();
            let mut with_citations = HashMap::new();
            for (name, with_biber) in notes {
                note_names.push(name.clone());
                with_citations.insert(name, with_biber);
            }
            let notes_with_biber = with_citations.values().filter(|v| **v).count();

            if note_names.is_empty() && projects.is_empty() {
                println!(
                    "{}",
                    tr!(
                        "No hay elementos pendientes de renderizado.",
                        "No items pending render."
                    )
                );
                return Ok(());
            }

            let output_dir = html_output_dir(paths);
            let output_dir_str = output_dir.to_string_lossy().to_string();
            println!(
                "{}: notas={} | proyectos={} | workers={} | formato={} | motor={} | pasadas_notas=1/2 | pasadas_proyectos=2 | con_biber={} | salida={}",
                tr("Plan render_updates", "Render updates plan"),
                note_names.len(),
                projects.len(),
                workers.max(1),
                format,
                render_motor(format)?,
                notes_with_biber,
                output_dir.display()
            );

            let paths_notes = paths.clone();
            let output_dir_notes = output_dir_str.clone();
            let db_notes = db.clone();
            run_parallel_render_with_progress(
                tr("Render updates · notas", "Render updates · notes"),
                note_names.clone(),
                workers,
                move |name| {
                    render_note_html_single_pass(&paths_notes, name)?;
                    if with_citations.get(name).copied().unwrap_or(false) {
                        run_biber_cmd(&paths_notes, name, Some(output_dir_notes.as_str()))?;
                        render_note_html_single_pass(&paths_notes, name)?;
                    }
                    let now = Utc::now();
                    run_with_sqlite_lock_retry("update note last_build_date_html", || {
                        db_notes
                            .lock()
                            .unwrap()
                            .set_note_last_build_date_html(name, now)
                            .map_err(anyhow::Error::from)
                    })?;
                    Ok(())
                },
            )?;

            let paths_projects = paths.clone();
            let output_dir_projects = output_dir_str.clone();
            let db_projects = db.clone();
            run_parallel_render_with_progress(
                tr("Render updates · proyectos", "Render updates · projects"),
                projects.clone(),
                workers,
                move |name| {
                    render_project_html_single_pass(&paths_projects, name)?;
                    run_biber_project_cmd(
                        &paths_projects,
                        name,
                        Some(output_dir_projects.as_str()),
                    )?;
                    render_project_html_single_pass(&paths_projects, name)?;
                    let now = Utc::now();
                    run_with_sqlite_lock_retry("update project last_build_date_html", || {
                        db_projects
                            .lock()
                            .unwrap()
                            .set_project_last_build_date_html(name, now)
                            .map_err(anyhow::Error::from)
                    })?;
                    Ok(())
                },
            )?;

            postprocess_html_output(paths)?;

            Ok(())
        }
        _ => bail!(tr!(
            "Formato no soportado: {format}",
            "Unsupported format: {format}"
        )),
    }
}

/// Motor de compilacion para cada formato. Fuente unica de verdad del plan de
/// render (los pasos reales usan el mismo criterio).
fn render_motor(format: &str) -> Result<&'static str> {
    match format {
        "pdf" => Ok("pdflatex"),
        "html" => Ok("make4ht"),
        other => bail!(tr!(
            "Formato no soportado: {other}",
            "Unsupported format: {other}"
        )),
    }
}

/// Pasadas del motor para una unidad segun formato y bibliografia:
/// - pdf: 2 pasadas de pdflatex; con biber, una tercera tras biber.
/// - html: 2 pasadas de make4ht (biber se ejecuta entre medias si aplica).
///
/// La excepcion es `render_updates` en html, cuyo plan es condicional por nota.
fn render_pass_count(format: &str, with_biber: bool) -> Result<usize> {
    Ok(match format {
        "pdf" => {
            if with_biber {
                3
            } else {
                2
            }
        }
        "html" => 2,
        other => bail!(tr!(
            "Formato no soportado: {other}",
            "Unsupported format: {other}"
        )),
    })
}

fn ztx_temp_dir(base: &std::path::Path) -> Result<std::path::PathBuf> {
    let dir = base.join(".zetteltex-tmp");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Extrae los nombres de notas destino referenciados por una nota, sin incluir
/// autoreferencias. Fuente compartida del lookup individual y del lote.
fn referenced_targets_from_note(path: &Path, source_note: &str) -> Result<BTreeSet<String>> {
    static EXCREF_NO_LABEL_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"\\excref\{([^}]+)\}").expect("regex excref valida")
    });

    let content = fs::read_to_string(path)?;
    let parsed = parse_note(&content);

    let mut targets = BTreeSet::new();
    for reference in &parsed.references {
        if reference.target_note != source_note {
            targets.insert(reference.target_note.clone());
        }
    }
    for caps in EXCREF_NO_LABEL_RE.captures_iter(&content) {
        if let Some(m) = caps.get(1) {
            let target = m.as_str().trim().to_string();
            if !target.is_empty() && target != source_note {
                targets.insert(target);
            }
        }
    }

    Ok(targets)
}

fn build_incoming_references_index(
    paths: &WorkspacePaths,
) -> Result<HashMap<String, Vec<(String, String)>>> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let mut index: HashMap<String, BTreeSet<(String, String)>> = HashMap::new();

    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        let Some(source_note) = note_stem_from_path(&path) else {
            continue;
        };

        let targets = referenced_targets_from_note(&path, &source_note)?;
        if targets.is_empty() {
            continue;
        }
        let title = db
            .note_title_by_filename(&source_note)?
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| source_note.clone());
        for target in targets {
            index
                .entry(target)
                .or_default()
                .insert((source_note.clone(), title.clone()));
        }
    }

    Ok(index
        .into_iter()
        .map(|(target, refs)| (target, refs.into_iter().collect()))
        .collect())
}

fn notes_referencing_target(
    paths: &WorkspacePaths,
    target_note: &str,
) -> Result<Vec<(String, String)>> {
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let mut refs = BTreeSet::new();

    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        let Some(source_note) = note_stem_from_path(&path) else {
            continue;
        };

        let targets = referenced_targets_from_note(&path, &source_note)?;
        if source_note != target_note && targets.contains(target_note) {
            let title = db
                .note_title_by_filename(&source_note)?
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| source_note.clone());
            refs.insert((source_note, title));
        }
    }

    Ok(refs.into_iter().collect())
}

fn inject_referenced_in_section(note_content: &str, incoming_notes: &[(String, String)]) -> String {
    if incoming_notes.is_empty() {
        return note_content.to_string();
    }

    let mut section = String::new();
    section.push_str("\n\\section*{Referenciado en}\n");
    section.push_str("\\begin{itemize}\n");
    for (note, title) in incoming_notes {
        // Link directly to the external anchor for each note's \currentdoc{note} label.
        section.push_str("  \\item ");
        section.push_str("\\hyperref[");
        section.push_str(note);
        section.push_str("-note]{");
        section.push_str(title);
        section.push_str("} ");
        section.push_str("\\ifx\\HCode\\UnDeFiNeD ");
        section.push_str("\\else ");
        section.push_str(title);
        section.push(' ');
        section.push_str("\\fi\n");
    }
    section.push_str("\\end{itemize}\n");

    if let Some(idx) = note_content.rfind("\\end{document}") {
        let mut out = String::with_capacity(note_content.len() + section.len());
        out.push_str(&note_content[..idx]);
        out.push_str(&section);
        out.push_str(&note_content[idx..]);
        out
    } else {
        let mut out = String::with_capacity(note_content.len() + section.len());
        out.push_str(note_content);
        out.push_str(&section);
        out
    }
}

pub(crate) fn run_biber_cmd(
    paths: &WorkspacePaths,
    name: &str,
    folder: Option<&str>,
) -> Result<()> {
    let output_dir = resolve_biber_folder(paths, folder);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;

    run_external_tool(
        "biber",
        &[
            &format!("--output-directory={}", output_dir.display()),
            name,
        ],
        Some(&paths.notes_slipbox),
        Some(load_zetteltex_config(paths).render.tool_timeout()),
    )
}

pub(crate) fn run_biber_project_cmd(
    paths: &WorkspacePaths,
    name: &str,
    folder: Option<&str>,
) -> Result<()> {
    let output_dir = resolve_biber_folder(paths, folder);
    let project_dir = paths.projects.join(name);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;

    run_external_tool(
        "biber",
        &[
            &format!("--output-directory={}", output_dir.display()),
            name,
        ],
        Some(&project_dir),
        Some(load_zetteltex_config(paths).render.tool_timeout()),
    )
}

fn resolve_biber_folder(paths: &WorkspacePaths, folder: Option<&str>) -> PathBuf {
    match folder {
        Some(raw) if !raw.is_empty() => {
            let candidate = PathBuf::from(raw);
            if candidate.is_absolute() {
                candidate
            } else {
                paths.root.join(candidate)
            }
        }
        _ => pdf_output_dir(paths),
    }
}
