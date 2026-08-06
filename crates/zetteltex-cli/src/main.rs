use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::OnceLock;
use std::sync::{mpsc, Arc, Mutex};
use std::{collections::{BTreeSet, HashMap}, fs};
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use clap::Parser;
use chrono::{Local, Utc};
use crossterm::cursor::Show;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size as terminal_size, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use regex::Regex;
// serde imports moved to module files that need them
use zetteltex_core::WorkspacePaths;
use zetteltex_db::init_database;
use zetteltex_parser::parse_note;
use tracing::{error, warn};

const DEFAULT_RECENT_FILES: usize = 10;
const DEFAULT_RENAME_RECENT_INDEX: usize = 1;
const DEFAULT_RENDER_WORKERS: usize = 4;

mod fuzzy;
use fuzzy::*;
mod export;
use export::*;
mod render;
use render::*;
mod sync;
use sync::*;
mod util;
use util::*;
mod cli;
use cli::*;
mod workspace;
use workspace::*;
mod notes;
use notes::*;
mod maintenance;
use maintenance::*;
mod rename;
use rename::*;
mod html;
use html::*;
mod ui;
use ui::*;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter("warn")
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    if let Some(Commands::Init) = &cli.command {
        return match init_workspace(&cli.workspace_root) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error inicializando workspace: {e}");
                ExitCode::from(1)
            }
        };
    }

    let paths = match WorkspacePaths::discover(&cli.workspace_root) {
        Ok(paths) => paths,
        Err(e) => {
            error!("{e}");
            eprintln!("Error de workspace: {e}");
            return ExitCode::from(2);
        }
    };

    match cli.command {
        None => {
            println!("zetteltex: usa --help para ver comandos disponibles");
            ExitCode::SUCCESS
        }
        Some(command) => match run_command(command, &paths) {
            Ok(code) => code,
            Err(e) => {
                error!("{e}");
                eprintln!("Error: {e}");
                ExitCode::from(1)
            }
        },
    }
}

fn run_command(command: Commands, paths: &WorkspacePaths) -> Result<ExitCode> {
    match command {
        Commands::Init => {
            // Este comando ya fue manejado en `main()` antes de cargar los paths,
            // pero Rust requiere que el pattern matching sea exhaustivo.
            Ok(ExitCode::SUCCESS)
        }
        Commands::InitConfig => init_config_interactive(paths),
        Commands::RenameNote { name } => {
            rename_note(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RemoveNote { name } => {
            remove_note(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::AddToDocuments { name } => {
            add_to_documents(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Newproject { name } => {
            create_project(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Newnote { name } => {
            create_note(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ListRecentFiles { n } => {
            list_recent_files(paths, n.unwrap_or(DEFAULT_RECENT_FILES))?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ListUnreferenced => {
            list_unreferenced(paths)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenameRecent { n } => {
            rename_recent(paths, n.unwrap_or(DEFAULT_RENAME_RECENT_INDEX))?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ListCitations { name } => {
            list_citations(paths, &name)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportProject { folder, texfile } => {
            export_project(paths, &folder, texfile.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportDraft {
            input_file,
            output_file,
        } => {
            export_draft(paths, &input_file, &output_file)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ToMd { note } => {
            to_md(paths, &note)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportMarkdown { note } => {
            export_markdown(paths, &note)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportProjectMarkdown { project } => {
            export_project_markdown(paths, &project)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportAllNotesMarkdown => {
            export_all_notes_markdown(paths)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportAllProjectsMarkdown => {
            export_all_projects_markdown(paths)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::ExportAllMarkdown => {
            export_all_markdown(paths)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Render {
            name,
            format,
            biber,
        } => {
            render::render_note_cmd(
                paths,
                &name,
                format.as_deref().unwrap_or("pdf"),
                biber.unwrap_or(false),
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenderProject {
            name,
            format,
            biber,
        } => {
            render::render_project_cmd(
                paths,
                &name,
                format.as_deref().unwrap_or("pdf"),
                biber.unwrap_or(false),
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenderAll { format, workers } => {
            render::render_all_notes_cmd(
                paths,
                format.as_deref().unwrap_or("pdf"),
                workers.unwrap_or(DEFAULT_RENDER_WORKERS),
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenderAllPdf { workers } => {
            render::render_all_notes_cmd(paths, "pdf", workers.unwrap_or(DEFAULT_RENDER_WORKERS))?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenderAllProjects { format, workers } => {
            render::render_all_projects_cmd(
                paths,
                format.as_deref().unwrap_or("pdf"),
                workers.unwrap_or(DEFAULT_RENDER_WORKERS),
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RenderUpdates { format, workers } => {
            render::render_updates_cmd(
                paths,
                format.as_deref().unwrap_or("pdf"),
                workers.unwrap_or(DEFAULT_RENDER_WORKERS),
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Biber { name, folder } => {
            run_biber_cmd(paths, &name, folder.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::BiberProject { name, folder } => {
            run_biber_project_cmd(paths, &name, folder.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::RemoveDuplicateCitations => {
            remove_duplicate_citations_cmd(paths)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Edit { name } => {
            edit_cmd(paths, name.as_deref())?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Fuzzy {
            inline,
            action,
            query,
            item,
            clipboard_text,
        } => {
            fuzzy_cmd(
                paths,
                inline,
                action.as_deref(),
                query.as_deref(),
                item.as_deref(),
                clipboard_text,
            )?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Clean => {
            let removed = clean_cmd(paths)?;
            println!("Clean summary: {} pdf(s), {} markdown(s) removed", removed.0, removed.1);
            Ok(ExitCode::SUCCESS)
        }
        Commands::Synchronize => {
            let note_stats = synchronize_notes(paths)?;
            let project_stats = synchronize_projects(paths)?;
            println!(
                "Sincronizacion completa: {} nota(s), {} link(s), {} referencia(s) sin resolver, {} proyecto(s), {} inclusion(es), {} inclusion(es) sin nota",
                note_stats.notes_synced,
                note_stats.links_built,
                note_stats.unresolved_references,
                project_stats.projects_synced,
                project_stats.inclusions_synced,
                project_stats.missing_notes
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::ForceSynchronizeNotes => {
            let stats = synchronize_notes(paths)?;
            println!(
                "Force synchronize notas: {} nota(s), {} link(s), {} referencia(s) sin resolver",
                stats.notes_synced, stats.links_built, stats.unresolved_references
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::ForceSynchronizeProjects => {
            let stats = synchronize_projects(paths)?;
            println!(
                "Force synchronize proyectos: {} proyecto(s), {} inclusion(es), {} inclusion(es) sin nota",
                stats.projects_synced, stats.inclusions_synced, stats.missing_notes
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::ForceSynchronize => {
            let note_stats = synchronize_notes(paths)?;
            let project_stats = synchronize_projects(paths)?;
            println!(
                "Force synchronize completo: {} nota(s), {} proyecto(s)",
                note_stats.notes_synced, project_stats.projects_synced
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::ListProjects => list_projects_cmd(paths),
        Commands::ListProjectInclusions { project } => {
            list_project_inclusions_cmd(paths, &project)
        }
        Commands::ListNoteProjects { note } => list_note_projects_cmd(paths, &note),
        Commands::ValidateReferences { notes_only, projects_only } => {
            let scope = if notes_only {
                ValidationScope::Notes
            } else if projects_only {
                ValidationScope::Projects
            } else {
                ValidationScope::Both
            };

            if scope != ValidationScope::Projects {
                let _ = synchronize_notes(paths)?;
            }
            let issues = validate_references(paths, scope)?;

            if issues.is_empty() {
                println!("✓ Todas las referencias son validas");
                return Ok(ExitCode::SUCCESS);
            }

            println!("Se encontraron {} referencia(s) rota(s):", issues.len());
            for issue in issues {
                println!(
                    "- [{}] {} -> {} [{}]",
                    issue.kind, issue.source, issue.target_note, issue.target_label
                );
            }
            Ok(ExitCode::from(1))
        }
    }
}


fn fuzzy_cmd(
    paths: &WorkspacePaths,
    inline: bool,
    action: Option<&str>,
    query: Option<&str>,
    item: Option<&str>,
    clipboard_text: Option<String>,
) -> Result<()> {
    if let Some(action_name) = action {
        return run_fuzzy_scripted_action(paths, action_name, query, item, clipboard_text);
    }

    if inline {
        return run_fuzzy_inline(paths);
    }

    launch_fuzzy_in_new_terminal(paths)
}

fn run_fuzzy_scripted_action(
    paths: &WorkspacePaths,
    action_name: &str,
    query: Option<&str>,
    item: Option<&str>,
    clipboard_text: Option<String>,
) -> Result<()> {
    let index = build_fuzzy_index(paths)?;

    let action = match action_name {
        "copy-exhyperref" => {
            let item = resolve_scripted_item(&index, query, item)?;
            FuzzyUiAction::CopyExhyperref { item }
        }
        "copy-excref" => {
            let item = resolve_scripted_item(&index, query, item)?;
            FuzzyUiAction::CopyExcref { item }
        }
        "open-editor" => {
            let item = resolve_scripted_item(&index, query, item)?;
            FuzzyUiAction::OpenEditor { item }
        }
        "open-pdf" => {
            let item = resolve_scripted_item(&index, query, item)?;
            FuzzyUiAction::OpenPdf { item }
        }
        "create-from-query" => FuzzyUiAction::CreateFromQuery {
            query: query.unwrap_or_default().to_string(),
        },
        "create-from-clipboard" => FuzzyUiAction::CreateFromClipboard,
        "copy-transclude" => {
            let item = resolve_scripted_item(&index, query, item)?;
            FuzzyUiAction::CopyTransclude { item }
        }
        other => {
            bail!(
                "Accion fuzzy no reconocida: {} (usa copy-exhyperref|copy-excref|open-editor|open-pdf|create-from-query|create-from-clipboard)",
                other
            )
        }
    };

    run_fuzzy_action(paths, &index, action, clipboard_text)
}

fn resolve_scripted_item(index: &FuzzyIndex, query: Option<&str>, item: Option<&str>) -> Result<FuzzyItem> {
    if let Some(target) = item {
        if let Some(found) = index
            .items
            .iter()
            .find(|i| i.display == target || i.name == target)
        {
            return Ok(found.clone());
        }
        bail!("No se encontro item fuzzy: {}", target)
    }

    if let Some(q) = query {
        let results = fuzzy_search(index, q, 1);
        if let Some((first, _)) = results.into_iter().next() {
            return Ok(first.clone());
        }
        bail!("No hay resultados fuzzy para query: {}", q)
    }

    bail!("Debes pasar --item o --query para acciones fuzzy scripted")
}

fn run_fuzzy_inline(paths: &WorkspacePaths) -> Result<()> {
    let index = build_fuzzy_index(paths)?;

    if index.items.is_empty() {
        println!("No hay notas ni proyectos para fuzzy.");
        return Ok(());
    }

    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        let action = run_fuzzy_tui(paths, &index)?;
        if let Some(action) = action {
            run_fuzzy_action(paths, &index, action, None)?;
        }
        return Ok(());
    }

    println!("Fuzzy inline (motor nativo Rust - fase de indexado)");
    println!("Escribe un termino y presiona Enter (linea vacia para salir).\n");

    loop {
        print!("fuzzy> ");
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let query = line.trim();

        if query.is_empty() {
            break;
        }

        let results = fuzzy_search(&index, query, index.settings.max_results);
        if results.is_empty() {
            println!("Sin resultados.\n");
            continue;
        }

        for (idx, (item, score)) in results.iter().take(10).enumerate() {
            println!("{}. {} ({:.1})", idx + 1, item.display, score);
        }
        println!();
    }

    Ok(())
}


fn run_fuzzy_action(
    paths: &WorkspacePaths,
    index: &FuzzyIndex,
    action: FuzzyUiAction,
    clipboard_override: Option<String>,
) -> Result<()> {
    match action {
        FuzzyUiAction::CopyExhyperref { item } => {
            let text = build_exhyperref_for_item(paths, index, &item)?;
            write_xclip_clipboard(&text)?;
            save_history_entry(paths, &item.display)?;
        }
        FuzzyUiAction::CopyExcref { item } => {
            let text = build_excref_for_item(paths, index, &item)?;
            write_xclip_clipboard(&text)?;
            save_history_entry(paths, &item.display)?;
        }
        FuzzyUiAction::OpenEditor { item } => {
            if item.kind == FuzzyItemKind::Project {
                let path = paths.projects.join(&item.name);
                open_in_editor(paths, &path)?;
            } else {
                let path = paths.notes_slipbox.join(format!("{}.tex", item.name));
                open_in_editor(paths, &path)?;
            }
            save_history_entry(paths, &item.display)?;
        }
        FuzzyUiAction::OpenPdf { item } => {
            open_pdf_best_effort(paths, &item.name)?;
            save_history_entry(paths, &item.display)?;
        }
        FuzzyUiAction::CreateFromQuery { query } => {
            let name = normalize_new_note_name(&query)?;
            create_note(paths, &name)?;
            let note_path = paths.notes_slipbox.join(format!("{}.tex", name));
            open_in_editor(paths, &note_path)?;
            save_history_entry(paths, &name)?;
        }
        FuzzyUiAction::CreateFromClipboard => {
            let content = clipboard_override.unwrap_or(read_xclip_clipboard()?);
            let name = note_name_from_clipboard_label(&content)?;
            create_note(paths, &name)?;
            let note_path = paths.notes_slipbox.join(format!("{}.tex", name));
            inject_clipboard_into_note_template(&note_path, &content)?;
            replace_note_today_date(&note_path)?;
            open_in_editor(paths, &note_path)?;
            write_xclip_clipboard(&format!(r"\transclude{{{}}}", name))?;
            save_history_entry(paths, &name)?;
        }
        FuzzyUiAction::CopyTransclude { item } => {
            let text = format!(r"\transclude{{{}}}", item.name);
            write_xclip_clipboard(&text)?;
            save_history_entry(paths, &item.display)?;
        }
    }
    Ok(())
}

fn fuzzy_pdf_candidate_paths(paths: &WorkspacePaths, item_name: &str) -> Vec<PathBuf> {
    vec![
        pdf_output_dir(paths).join(format!("{}.pdf", item_name)),
        paths
            .root
            .join("jabberwocky")
            .join("adjuntos")
            .join("pdf")
            .join(format!("{}.pdf", item_name)),
    ]
}

fn open_pdf_best_effort(paths: &WorkspacePaths, item_name: &str) -> Result<()> {
    let candidates = fuzzy_pdf_candidate_paths(paths, item_name);

    let chosen = candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone());

    if !chosen.exists() {
        bail!("PDF not found: {}", chosen.display());
    }

    let chosen_str = chosen.to_string_lossy();
    if let Ok(custom) = std::env::var("ZETTELTEX_PDF_OPENER") {
        let trimmed = custom.trim();
        if !trimmed.is_empty()
            && run_external_open_nonblocking_verified(trimmed, &[chosen_str.as_ref()], None).is_ok()
        {
            return Ok(());
        }
    }

    let direct_openers = [
        "qpdfview",
        "zathura",
        "okular",
        "evince",
        "atril",
        "mupdf",
        "/usr/bin/qpdfview",
        "/usr/bin/zathura",
        "/usr/bin/okular",
        "/usr/bin/evince",
        "/usr/bin/atril",
        "/usr/bin/mupdf",
    ];
    for opener in direct_openers {
        if opener == "qpdfview" {
            // Prefer --unique for qpdfview to reuse existing window in tests
            if run_external_open_nonblocking_verified(opener, &["--unique", chosen_str.as_ref()], None).is_ok() {
                return Ok(());
            }
        }
        if run_external_open_nonblocking_verified(opener, &[chosen_str.as_ref()], None).is_ok() {
            return Ok(());
        }
    }

    if run_external_open_nonblocking_verified("xdg-open", &[chosen_str.as_ref()], None).is_ok() {
        return Ok(());
    }
    if run_external_open_nonblocking_verified("/usr/bin/xdg-open", &[chosen_str.as_ref()], None).is_ok() {
        return Ok(());
    }
    if run_external_open_nonblocking_verified("gio", &["open", chosen_str.as_ref()], None).is_ok() {
        return Ok(());
    }
    if run_external_open_nonblocking_verified("/usr/bin/gio", &["open", chosen_str.as_ref()], None).is_ok() {
        return Ok(());
    }

    bail!(
        "No se pudo abrir el PDF con ningun visor candidato (custom/directo/xdg-open/gio): {}",
        chosen.display()
    )
}

fn normalize_new_note_name(raw: &str) -> Result<String> {
    let mut name = raw.trim().to_string();
    if name.to_lowercase().ends_with(".tex") {
        name.truncate(name.len() - 4);
    }
    name = name.replace([':', ' '], "-");
    if name.is_empty() {
        bail!("No se puede crear una nota sin nombre")
    }
    Ok(name)
}

fn note_name_from_clipboard_label(content: &str) -> Result<String> {
    let re = Regex::new(r"\\label\{([^}]+)\}")?;
    let caps = re
        .captures(content)
        .ok_or_else(|| anyhow::anyhow!("No se encontro ninguna etiqueta \\label{{...}} en el portapapeles"))?;
    let mut label = caps
        .get(1)
        .map(|m| m.as_str().trim().to_string())
        .unwrap_or_default();
    if let Some(rest) = label.strip_prefix("defn:") {
        label = rest.to_string();
    }
    label = label.replace(':', "-");
    if label.is_empty() {
        bail!("Etiqueta de portapapeles invalida")
    }
    Ok(label)
}

fn inject_clipboard_into_note_template(note_path: &Path, clipboard_content: &str) -> Result<()> {
    let original = fs::read_to_string(note_path)?;
    let indented = clipboard_content
        .lines()
        .map(|line| format!("    {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    let marker = "        %Write Note here";
    let updated = if original.contains(marker) {
        original.replace(marker, &indented)
    } else if let Some(pos) = original.find("\\end{document}") {
        format!("{}\n{}\n{}", &original[..pos], indented, &original[pos..])
    } else {
        format!("{}\n{}\n", original, indented)
    };

    fs::write(note_path, updated)?;
    Ok(())
}

fn replace_note_today_date(note_path: &Path) -> Result<()> {
    let original = fs::read_to_string(note_path)?;
    let today = Local::now().format("%d/%m/%Y").to_string();
    let updated = original.replace("\\date{\\today}", &format!("\\date{{{today}}}"));
    fs::write(note_path, updated)?;
    Ok(())
}


