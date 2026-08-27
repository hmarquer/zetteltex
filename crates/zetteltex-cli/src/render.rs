use super::*;
use crate::i18n::tr;

pub(crate) fn render_note_cmd(
    paths: &WorkspacePaths,
    name: &str,
    format: &str,
    with_biber: bool,
) -> Result<()> {
    let auto_biber = with_biber || note_contains_citations(paths, name)?;
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

            render_note_pdf(paths, name, auto_biber)?;

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

            render_note_html_single_pass(paths, name)?;

            if auto_biber {
                let output_dir_str = output_dir.to_string_lossy().to_string();
                run_biber_cmd(paths, name, Some(output_dir_str.as_str()))?;
            }

            render_note_html_single_pass(paths, name)?;

            postprocess_html_output(paths)?;

            let db = init_database(&paths.root.join("slipbox.db"))?;
            db.set_note_last_build_date_html(name, Utc::now())?;
            Ok(())
        }
        _ => bail!(tr!("Formato no soportado: {format}", "Unsupported format: {format}")),
    }
}

pub(crate) fn render_project_cmd(
    paths: &WorkspacePaths,
    name: &str,
    format: &str,
    with_biber: bool,
) -> Result<()> {
    let auto_biber = with_biber || project_contains_citations(paths, name)?;
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

            render_project_pdf(paths, name, auto_biber)?;

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

            render_project_html_single_pass(paths, name)?;

            if auto_biber {
                let output_dir_str = output_dir.to_string_lossy().to_string();
                run_biber_project_cmd(paths, name, Some(output_dir_str.as_str()))?;
            }

            render_project_html_single_pass(paths, name)?;

            postprocess_html_output(paths)?;

            let db = init_database(&paths.root.join("slipbox.db"))?;
            db.set_project_last_build_date_html(name, Utc::now())?;
            Ok(())
        }
        _ => bail!(tr!("Formato no soportado: {format}", "Unsupported format: {format}")),
    }
}

pub(crate) fn render_all_notes_cmd(paths: &WorkspacePaths, format: &str, workers: usize) -> Result<()> {
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
                with_citations.insert(name.clone(), note_contains_citations(paths, name)?);
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
                with_citations.insert(name.clone(), note_contains_citations(paths, name)?);
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
        _ => bail!(tr!("Formato no soportado: {format}", "Unsupported format: {format}")),
    }
}

pub(crate) fn render_all_projects_cmd(paths: &WorkspacePaths, format: &str, workers: usize) -> Result<()> {
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
                with_citations.insert(name.clone(), project_contains_citations(paths, name)?);
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
                with_citations.insert(name.clone(), project_contains_citations(paths, name)?);
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
                tr("Render proyectos · pasada 1/2", "Render projects · pass 1/2"),
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
                tr("Render proyectos · pasada 2/2", "Render projects · pass 2/2"),
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
        _ => bail!(tr!("Formato no soportado: {format}", "Unsupported format: {format}")),
    }
}

pub(crate) fn render_updates_cmd(paths: &WorkspacePaths, format: &str, workers: usize) -> Result<()> {
    match format {
        "pdf" => {
            println!("{}", tr!("Preparando render_updates: sincronizando indices...", "Preparing render_updates: synchronizing indexes..."));
            let _ = run_with_sqlite_lock_retry("synchronize notes", || synchronize_notes(paths))?;
            let _ = run_with_sqlite_lock_retry("synchronize projects", || synchronize_projects(paths))?;

            let db = run_with_sqlite_lock_retry("open database", || {
                init_database(&paths.root.join("slipbox.db"))
            })?;
            let notes = db
                .notes_needing_render()?
                .into_iter()
                .filter(|n| !is_render_temp_note_name(n))
                .filter(|n| paths.notes_slipbox.join(format!("{n}.tex")).exists())
                .collect::<Vec<_>>();
            let projects = db.projects_needing_render()?;

            if notes.is_empty() && projects.is_empty() {
                println!("{}", tr!("No hay elementos pendientes de renderizado.", "No items pending render."));
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
            run_parallel_render_with_progress(
                tr("Render updates · notas", "Render updates · notes"),
                notes.clone(),
                workers,
                move |name| {
                    render_note_pdf(&paths_notes, name, false)?;
                    Ok(())
                },
            )?;

            for name in &notes {
                run_with_sqlite_lock_retry("update note last_build_date_pdf", || {
                    db.set_note_last_build_date_pdf(name, Utc::now())
                })?;
            }

            let paths_projects = paths.clone();
            run_parallel_render_with_progress(
                tr("Render updates · proyectos", "Render updates · projects"),
                projects.clone(),
                workers,
                move |name| {
                    render_project_pdf(&paths_projects, name, false)?;
                    Ok(())
                },
            )?;

            for name in &projects {
                run_with_sqlite_lock_retry("update project last_build_date_pdf", || {
                    db.set_project_last_build_date_pdf(name, Utc::now())
                })?;
            }

            Ok(())
        }
        "html" => {
            println!("{}", tr!("Preparando render_updates: sincronizando indices...", "Preparing render_updates: synchronizing indexes..."));
            let _ = run_with_sqlite_lock_retry("synchronize notes", || synchronize_notes(paths))?;
            let _ = run_with_sqlite_lock_retry("synchronize projects", || synchronize_projects(paths))?;

            let db = run_with_sqlite_lock_retry("open database", || {
                init_database(&paths.root.join("slipbox.db"))
            })?;
            let notes = db
                .notes_needing_render_html()?
                .into_iter()
                .filter(|n| !is_render_temp_note_name(n))
                .filter(|n| paths.notes_slipbox.join(format!("{n}.tex")).exists())
                .map(|n| (n.clone(), db.note_has_citations(&n).unwrap_or(false)))
                .collect::<Vec<_>>();
            let projects = db.projects_needing_render_html()?;

            let mut note_names = Vec::new();
            let mut with_citations = HashMap::new();
            for (name, with_biber) in notes {
                note_names.push(name.clone());
                with_citations.insert(name, with_biber);
            }
            let notes_with_biber = with_citations.values().filter(|v| **v).count();

            if note_names.is_empty() && projects.is_empty() {
                println!("{}", tr!("No hay elementos pendientes de renderizado.", "No items pending render."));
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
                    Ok(())
                },
            )?;

            let paths_projects = paths.clone();
            let output_dir_projects = output_dir_str.clone();
            run_parallel_render_with_progress(
                tr("Render updates · proyectos", "Render updates · projects"),
                projects.clone(),
                workers,
                move |name| {
                    render_project_html_single_pass(&paths_projects, name)?;
                    run_biber_project_cmd(&paths_projects, name, Some(output_dir_projects.as_str()))?;
                    render_project_html_single_pass(&paths_projects, name)?;
                    Ok(())
                },
            )?;

            postprocess_html_output(paths)?;

            for name in &note_names {
                run_with_sqlite_lock_retry("update note last_build_date_html", || {
                    db.set_note_last_build_date_html(name, Utc::now())
                })?;
            }

            for name in &projects {
                run_with_sqlite_lock_retry("update project last_build_date_html", || {
                    db.set_project_last_build_date_html(name, Utc::now())
                })?;
            }

            Ok(())
        }
        _ => bail!(tr!("Formato no soportado: {format}", "Unsupported format: {format}")),
    }
}

fn run_with_sqlite_lock_retry<T, F>(label: &str, mut operation: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    const MAX_ATTEMPTS: usize = 8;

    for attempt in 1..=MAX_ATTEMPTS {
        match operation() {
            Ok(value) => return Ok(value),
            Err(err) => {
                let retryable = is_sqlite_lock_error(&err);
                if retryable && attempt < MAX_ATTEMPTS {
                    let backoff_ms = 200_u64 * attempt as u64;
                    warn!(
                        "{} hit sqlite lock (attempt {}/{}), retrying in {}ms",
                        label, attempt, MAX_ATTEMPTS, backoff_ms
                    );
                    std::thread::sleep(Duration::from_millis(backoff_ms));
                    continue;
                }
                return Err(err);
            }
        }
    }

    bail!("{}", tr!("{label} fallo despues de reintentos", "{label} failed after retries"))
}

fn is_sqlite_lock_error(err: &anyhow::Error) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("database is locked")
        || msg.contains("database table is locked")
        || msg.contains("database busy")
}

#[derive(Debug)]
enum RenderEvent {
    Started(String),
    Finished(String),
    Failed { file: String, error: String },
}

#[derive(Debug, Clone, Copy)]
struct ProgressLineLayout {
    max_cols: usize,
    file_width: usize,
    bar_width: usize,
    counter_digits: usize,
}

fn run_parallel_render_with_progress<F>(
    phase_label: &str,
    items: Vec<String>,
    workers: usize,
    job: F,
) -> Result<()>
where
    F: Fn(&str) -> Result<()> + Send + Sync + 'static,
{
    if items.is_empty() {
        return Ok(());
    }

    let worker_count = workers.max(1).min(items.len());
    let total = items.len() as u64;
    let use_tty_progress = std::io::stdout().is_terminal();
    let progress_layout = if use_tty_progress {
        Some(build_progress_line_layout(total))
    } else {
        None
    };

    let queue = Arc::new(Mutex::new(items));
    let job = Arc::new(job);
    let (event_tx, event_rx) = mpsc::channel::<RenderEvent>();

    let mut handles = Vec::with_capacity(worker_count);
    for _ in 0..worker_count {
        let queue = Arc::clone(&queue);
        let job = Arc::clone(&job);
        let event_tx = event_tx.clone();
        handles.push(std::thread::spawn(move || {
            loop {
                let next = {
                    let mut guard = queue.lock().expect("render queue lock poisoned");
                    guard.pop()
                };

                let Some(file) = next else {
                    break;
                };

                let _ = event_tx.send(RenderEvent::Started(file.clone()));
                match job(&file) {
                    Ok(()) => {
                        let _ = event_tx.send(RenderEvent::Finished(file));
                    }
                    Err(err) => {
                        let _ = event_tx.send(RenderEvent::Failed {
                            file,
                            error: err.to_string(),
                        });
                    }
                }
            }
        }));
    }
    drop(event_tx);

    let mut completed = 0usize;
    let mut active = std::collections::HashSet::new();
    let mut errors = Vec::new();
    let started_at = Instant::now();
    let mut current_file = String::from("-");
    let mut smoothed_secs_per_item: Option<f64> = None;
    let mut eta_anchor_secs: u64 = 0;
    let mut eta_anchor_at = Instant::now();

    if use_tty_progress {
        render_compact_progress_line(
            progress_layout.as_ref().expect("layout must exist"),
            &current_file,
            0,
            total,
            started_at.elapsed(),
            Duration::from_secs(0),
        )?;
    }

    while completed < total as usize {
        match event_rx.recv_timeout(Duration::from_millis(120)) {
            Ok(RenderEvent::Started(file)) => {
                current_file = file.clone();
                active.insert(file);
                if use_tty_progress {
                    let elapsed = started_at.elapsed();
                    let eta = eta_anchor_secs.saturating_sub(eta_anchor_at.elapsed().as_secs());
                    render_compact_progress_line(
                        progress_layout.as_ref().expect("layout must exist"),
                        &current_file,
                        completed as u64,
                        total,
                        elapsed,
                        Duration::from_secs(eta),
                    )?;
                }
            }
            Ok(RenderEvent::Finished(file)) => {
                active.remove(&file);
                completed += 1;
                if !use_tty_progress {
                    println!("{}: {}/{}", phase_label, completed, total);
                }
                current_file = if active.is_empty() {
                    file
                } else {
                    active.iter().next().cloned().unwrap_or_else(|| "-".to_string())
                };

                // ETA suavizado: EMA de segundos por item sobre throughput global,
                // y luego cuenta regresiva entre eventos para evitar dientes de sierra.
                let elapsed = started_at.elapsed();
                let current_secs_per_item = elapsed.as_secs_f64() / completed as f64;
                smoothed_secs_per_item = Some(match smoothed_secs_per_item {
                    Some(prev) => (0.22 * current_secs_per_item) + (0.78 * prev),
                    None => current_secs_per_item,
                });
                let remaining = (total as usize).saturating_sub(completed) as f64;
                let eta_estimated = smoothed_secs_per_item.unwrap_or(0.0) * remaining;
                eta_anchor_secs = eta_estimated.max(0.0).round() as u64;
                eta_anchor_at = Instant::now();

                if use_tty_progress {
                    render_compact_progress_line(
                        progress_layout.as_ref().expect("layout must exist"),
                        &current_file,
                        completed as u64,
                        total,
                        elapsed,
                        Duration::from_secs(eta_anchor_secs),
                    )?;
                }
            }
            Ok(RenderEvent::Failed { file, error }) => {
                active.remove(&file);
                completed += 1;
                if !use_tty_progress {
                    println!("{}: {}/{}", phase_label, completed, total);
                }
                current_file = if active.is_empty() {
                    file.clone()
                } else {
                    active.iter().next().cloned().unwrap_or_else(|| "-".to_string())
                };

                let elapsed = started_at.elapsed();
                let current_secs_per_item = elapsed.as_secs_f64() / completed as f64;
                smoothed_secs_per_item = Some(match smoothed_secs_per_item {
                    Some(prev) => (0.22 * current_secs_per_item) + (0.78 * prev),
                    None => current_secs_per_item,
                });
                let remaining = (total as usize).saturating_sub(completed) as f64;
                let eta_estimated = smoothed_secs_per_item.unwrap_or(0.0) * remaining;
                eta_anchor_secs = eta_estimated.max(0.0).round() as u64;
                eta_anchor_at = Instant::now();

                if use_tty_progress {
                    render_compact_progress_line(
                        progress_layout.as_ref().expect("layout must exist"),
                        &current_file,
                        completed as u64,
                        total,
                        elapsed,
                        Duration::from_secs(eta_anchor_secs),
                    )?;
                }
                errors.push(format!("{}: {}", file, error));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if use_tty_progress {
                    let elapsed = started_at.elapsed();
                    let eta = eta_anchor_secs.saturating_sub(eta_anchor_at.elapsed().as_secs());
                    render_compact_progress_line(
                        progress_layout.as_ref().expect("layout must exist"),
                        &current_file,
                        completed as u64,
                        total,
                        elapsed,
                        Duration::from_secs(eta),
                    )?;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    if use_tty_progress {
        println!();
    }

    for handle in handles {
        let _ = handle.join();
    }

    if errors.is_empty() {
        println!("{} | {}", phase_label, tr("completado", "completed"));
        return Ok(());
    }

    errors.sort();
    let total_errors = errors.len();
    println!("{} | {}: {}", phase_label, tr("errores", "errors"), total_errors);
    for error in &errors {
        eprintln!("  - {}", error);
    }
    bail!(
        "{} {} {}",
        phase_label,
        tr("fallo en", "failed in"),
        tr!("{} archivo(s)", "{} file(s)", total_errors)
    )
}

fn render_compact_progress_line(
    layout: &ProgressLineLayout,
    current_file: &str,
    completed: u64,
    total: u64,
    elapsed: Duration,
    eta: Duration,
) -> Result<()> {
    let counter = format!(
        "{:0width$}/{:0width$}",
        completed,
        total,
        width = layout.counter_digits
    );
    let elapsed_s = format_hhmmss(elapsed.as_secs());
    let eta_s = format_hhmmss(eta.as_secs());

    let filled = if total == 0 {
        0
    } else {
        ((completed as usize) * layout.bar_width) / (total as usize)
    };
    let bar = format!(
        "{}{}",
        "#".repeat(filled),
        "-".repeat(layout.bar_width.saturating_sub(filled))
    );
    let file_short = fit_file_field(current_file, layout.file_width);
    let mut line = format!(
        "{} [{}] {} [{}/{}]",
        file_short, bar, counter, elapsed_s, eta_s
    );

    // Reserva siempre 1 columna para evitar autowrap al borde derecho,
    // que provoca salto de linea visual en muchos terminales.
    if line.chars().count() > layout.max_cols {
        line = line.chars().take(layout.max_cols).collect::<String>();
    }

    // Limpia la linea actual y reescribe sobre la misma.
    print!("\r\x1b[2K{}", line);
    io::stdout().flush()?;
    Ok(())
}

fn terminal_columns() -> usize {
    match terminal_size() {
        Ok((w, _)) => w as usize,
        Err(_) => 100,
    }
}

fn build_progress_line_layout(total: u64) -> ProgressLineLayout {
    let cols = terminal_columns().max(40);
    let max_cols = cols.saturating_sub(1);
    let counter_digits = total.to_string().len().max(1);

    // "0000/1105" -> 2*digits + 1
    let counter_width = (2 * counter_digits) + 1;
    // "[00:00:00/00:00:00]" -> 19
    let time_block_width = 19usize;
    // Separadores fijos en: "<file> [<bar>] <counter> <time>"
    let separators_width = 5usize;

    let mut available_for_file_and_bar =
        max_cols.saturating_sub(counter_width + time_block_width + separators_width);

    // Minimos para mantener legibilidad.
    if available_for_file_and_bar < 16 {
        available_for_file_and_bar = 16;
    }

    let min_file_width = 8usize;
    let min_bar_width = 8usize;

    let mut file_width = (available_for_file_and_bar / 3).clamp(min_file_width, 36);
    let mut bar_width = available_for_file_and_bar.saturating_sub(file_width);

    if bar_width < min_bar_width {
        let delta = min_bar_width - bar_width;
        file_width = file_width.saturating_sub(delta);
        bar_width = available_for_file_and_bar.saturating_sub(file_width);
    }

    if file_width < min_file_width {
        file_width = min_file_width;
        bar_width = available_for_file_and_bar.saturating_sub(file_width).max(min_bar_width);
    }

    ProgressLineLayout {
        max_cols,
        file_width,
        bar_width,
        counter_digits,
    }
}

fn fit_file_field(name: &str, width: usize) -> String {
    let mut s: String = name.chars().take(width).collect();
    let len = s.chars().count();
    if len < width {
        s.push_str(&" ".repeat(width - len));
    }
    s
}

fn format_hhmmss(total_secs: u64) -> String {
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// Motor de compilacion para cada formato. Fuente unica de verdad del plan de
/// render (los pasos reales usan el mismo criterio).
fn render_motor(format: &str) -> Result<&'static str> {
    match format {
        "pdf" => Ok("pdflatex"),
        "html" => Ok("make4ht"),
        other => bail!(tr!("Formato no soportado: {other}", "Unsupported format: {other}")),
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
        other => bail!(tr!("Formato no soportado: {other}", "Unsupported format: {other}")),
    })
}

fn ztx_temp_dir(base: &std::path::Path) -> Result<std::path::PathBuf> {
    let dir = base.join(".zetteltex-tmp");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn render_note_pdf(paths: &WorkspacePaths, name: &str, with_biber: bool) -> Result<()> {
    let note_path = paths.notes_slipbox.join(format!("{name}.tex"));
    if !note_path.exists() {
        bail!("{}: {}", tr("El archivo no existe", "No such file"), note_path.display());
    }

    let output_dir = pdf_output_dir(paths);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;

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
    let temp_path_str = temp_path.to_string_lossy().to_string();

    let auto_biber = with_biber || note_contains_citations(paths, name)?;

    // pdflatex needs 2 passes for \label/\ref and a third one after biber to
    // settle biblatex's citations (with only 2 passes it leaves "Please rerun").
    run_pdflatex_pass(paths, name, temp_path_str.as_str(), &paths.notes_slipbox)?;
    if auto_biber {
        run_biber_cmd(paths, name, None)?;
    }
    run_pdflatex_pass(paths, name, temp_path_str.as_str(), &paths.notes_slipbox)?;
    if auto_biber {
        run_pdflatex_pass(paths, name, temp_path_str.as_str(), &paths.notes_slipbox)?;
    }

    // Keep the temp file for debugging when pdflatex fails.
    if let Err(err) = fs::remove_file(&temp_path) {
        return Err(err.into());
    }

    Ok(())
}

fn ensure_backlink_sources(paths: &WorkspacePaths, incoming_notes: &[(String, String)]) -> Result<()> {
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
            run_pdflatex_pass(paths, source, &tex_path.to_string_lossy(), &paths.notes_slipbox)?;
        }
    }
    Ok(())
}

fn run_pdflatex_pass(paths: &WorkspacePaths, name: &str, input_path: &str, cwd: &Path) -> Result<()> {
    let output_dir = pdf_output_dir(paths);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;
    run_external_tool(
        "pdflatex",
        &[
            "-interaction=nonstopmode",
            &format!("--jobname={name}"),
            "-shell-escape",
            &format!("-output-directory={}", output_dir.display()),
            input_path,
        ],
        Some(cwd),
    )
}

fn render_note_html_single_pass(paths: &WorkspacePaths, name: &str) -> Result<()> {
    let note_path = paths.notes_slipbox.join(format!("{name}.tex"));
    if !note_path.exists() {
        bail!("{}: {}", tr("El archivo no existe", "No such file"), note_path.display());
    }

    let output_dir = html_output_dir(paths);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;

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
    let temp_path_str = temp_path.to_string_lossy().to_string();

    let output_dir_str = output_dir.to_string_lossy().to_string();
    let render_result = run_external_tool(
        "make4ht",
        &[
            "--format",
            "html5+svg",
            "--output-dir",
            output_dir_str.as_str(),
            "--jobname",
            name,
            "--shell-escape",
            temp_path_str.as_str(),
            HTML_TEX4HT_MATH_OPTS,
        ],
        Some(&paths.notes_slipbox),
    );

    match render_result {
        Ok(_) => {
            fs::remove_file(&temp_path)?;
            fs::remove_file(&debug_path)?;
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn build_incoming_references_index(
    paths: &WorkspacePaths,
) -> Result<HashMap<String, Vec<(String, String)>>> {
    let excref_no_label_re = Regex::new(r"\\excref\{([^}]+)\}")?;
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let mut index: HashMap<String, BTreeSet<(String, String)>> = HashMap::new();

    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        let Some(source_note) = note_stem_from_path(&path) else {
            continue;
        };

        let content = fs::read_to_string(&path)?;
        let parsed = parse_note(&content)?;

        let mut targets = BTreeSet::new();
        for reference in &parsed.references {
            if reference.target_note != source_note {
                targets.insert(reference.target_note.clone());
            }
        }
        for caps in excref_no_label_re.captures_iter(&content) {
            if let Some(m) = caps.get(1) {
                let target = m.as_str().trim().to_string();
                if !target.is_empty() && target != source_note {
                    targets.insert(target);
                }
            }
        }

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

fn notes_referencing_target(paths: &WorkspacePaths, target_note: &str) -> Result<Vec<(String, String)>> {
    let excref_no_label_re = Regex::new(r"\\excref\{([^}]+)\}")?;
    let db = init_database(&paths.root.join("slipbox.db"))?;
    let mut refs = BTreeSet::new();

    for entry in fs::read_dir(&paths.notes_slipbox)? {
        let entry = entry?;
        let path = entry.path();
        let Some(source_note) = note_stem_from_path(&path) else {
            continue;
        };

        let content = fs::read_to_string(&path)?;
        let parsed = parse_note(&content)?;

        let via_structured_ref = parsed
            .references
            .iter()
            .any(|reference| reference.target_note == target_note);
        let via_excref_without_label = excref_no_label_re
            .captures_iter(&content)
            .any(|caps| caps.get(1).map(|m| m.as_str().trim()) == Some(target_note));

        if source_note != target_note && (via_structured_ref || via_excref_without_label) {
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

fn inject_html_overrides(note_content: &str) -> String {
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

fn render_project_pdf(paths: &WorkspacePaths, name: &str, with_biber: bool) -> Result<()> {
    let project_dir = paths.projects.join(name);
    let project_path = project_dir.join(format!("{name}.tex"));
    if !project_path.exists() {
        bail!("{}: {}", tr("El archivo no existe", "No such file"), project_path.display());
    }

    let file_name = project_path.file_name().unwrap().to_string_lossy();

    let auto_biber = with_biber || project_contains_citations(paths, name)?;

    // Same pass orchestration as notes: 2 pdflatex passes, plus a third one
    // after biber when the project cites entries from the bibliography.
    run_pdflatex_pass(paths, name, file_name.as_ref(), &project_dir)?;
    if auto_biber {
        run_biber_project_cmd(paths, name, None)?;
    }
    run_pdflatex_pass(paths, name, file_name.as_ref(), &project_dir)?;
    if auto_biber {
        run_pdflatex_pass(paths, name, file_name.as_ref(), &project_dir)?;
    }

    Ok(())
}

fn render_project_html_single_pass(paths: &WorkspacePaths, name: &str) -> Result<()> {
    let project_dir = paths.projects.join(name);
    let project_path = project_dir.join(format!("{name}.tex"));
    if !project_path.exists() {
        bail!("{}: {}", tr("El archivo no existe", "No such file"), project_path.display());
    }

    let output_dir = html_output_dir(paths);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;

    let file_name = project_path.file_name().unwrap().to_string_lossy();
    let output_dir_str = output_dir.to_string_lossy().to_string();

    run_external_tool(
        "make4ht",
        &[
            "--format",
            "html5+svg",
            "--output-dir",
            output_dir_str.as_str(),
            "--jobname",
            name,
            "--shell-escape",
            file_name.as_ref(),
            HTML_TEX4HT_MATH_OPTS,
        ],
        Some(&project_dir),
    )
}

fn note_contains_citations(paths: &WorkspacePaths, name: &str) -> Result<bool> {
    let note_path = paths.notes_slipbox.join(format!("{name}.tex"));
    let content = fs::read_to_string(note_path)?;
    let parsed = parse_note(&content)?;
    Ok(!parsed.citations.is_empty())
}

fn project_contains_citations(paths: &WorkspacePaths, name: &str) -> Result<bool> {
    let project_path = paths.projects.join(name).join(format!("{name}.tex"));
    let content = fs::read_to_string(project_path)?;
    let cite_re = Regex::new(r"\\(?:no)?cite[a-zA-Z\*]*\s*(?:\[[^\]]*\]\s*)?\{")?;
    Ok(cite_re.is_match(&content))
}

pub(crate) fn run_biber_cmd(paths: &WorkspacePaths, name: &str, folder: Option<&str>) -> Result<()> {
    let output_dir = resolve_biber_folder(paths, folder);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;
    
    run_external_tool(
        "biber", 
        &[
            &format!("--output-directory={}", output_dir.display()),
            name
        ], 
        Some(&paths.notes_slipbox)
    )
}

pub(crate) fn run_biber_project_cmd(paths: &WorkspacePaths, name: &str, folder: Option<&str>) -> Result<()> {
    let output_dir = resolve_biber_folder(paths, folder);
    let project_dir = paths.projects.join(name);
    fs::create_dir_all(&output_dir)?;
    let output_dir = fs::canonicalize(&output_dir)?;
    
    run_external_tool(
        "biber", 
        &[
            &format!("--output-directory={}", output_dir.display()),
            name
        ], 
        Some(&project_dir)
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

pub(crate) fn pdf_output_dir(paths: &WorkspacePaths) -> PathBuf {
    let config = load_zetteltex_config(paths);
    config
        .render
        .pdf_output_dir
        .as_deref()
        .map(|raw| resolve_config_path(&paths.root, raw))
        .unwrap_or_else(|| paths.root.join("pdf"))
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

const HTML_TEX4HT_MATH_OPTS: &str = "pic-m+,pic-equation,pic-eqnarray,pic-array,pic-matrix,pic-align,pic-cases";