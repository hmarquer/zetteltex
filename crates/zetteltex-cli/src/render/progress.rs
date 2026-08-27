use super::*;

#[derive(Debug)]
pub(crate) enum RenderEvent {
    Started(String),
    Finished(String),
    Failed { file: String, error: String },
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ProgressLineLayout {
    max_cols: usize,
    file_width: usize,
    bar_width: usize,
    counter_digits: usize,
}

/// Run a batch render across `workers` threads, reporting a single-line compact
/// progress bar on TTYs and a per-item counter otherwise.
pub(crate) fn run_parallel_render_with_progress<F>(
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
        handles.push(std::thread::spawn(move || loop {
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
                    active
                        .iter()
                        .next()
                        .cloned()
                        .unwrap_or_else(|| "-".to_string())
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
                    active
                        .iter()
                        .next()
                        .cloned()
                        .unwrap_or_else(|| "-".to_string())
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
    println!(
        "{} | {}: {}",
        phase_label,
        tr("errores", "errors"),
        total_errors
    );
    for error in &errors {
        warn!("{}", error);
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
        bar_width = available_for_file_and_bar
            .saturating_sub(file_width)
            .max(min_bar_width);
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
